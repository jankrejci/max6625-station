use crate::config::SensorDescriptor;
use crate::kalman::Kalman;
use crate::spi::Spi;
use anyhow::{anyhow, Context, Result};
use log::warn;
use rppal::gpio::{Gpio, OutputPin};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

const PROCESS_VARIANCE: f64 = 0.01;
const MEASUREMENT_ERROR: f64 = 2.0;

pub struct Temperatures {
    pub inner: BTreeMap<usize, f64>,
    pub filtered: BTreeMap<usize, Kalman>,
    pub calibration: BTreeMap<usize, f64>,
}

impl Temperatures {
    const DEFAULT_OFFSET: f64 = 0.0;

    pub fn new(num_sensors: usize) -> Self {
        let mut default_calibration = BTreeMap::new();
        let mut filtered = BTreeMap::new();
        for sensor_id in 0..num_sensors {
            // Default calibration offset is 0.0 ˚C
            default_calibration.insert(sensor_id, Self::DEFAULT_OFFSET);
            filtered.insert(sensor_id, Kalman::new(MEASUREMENT_ERROR, PROCESS_VARIANCE));
        }

        Self {
            inner: BTreeMap::new(),
            filtered,
            calibration: default_calibration,
        }
    }

    pub fn load_calibration(&mut self, path: &str) -> Result<()> {
        let calibration_file = File::open(path).context("Failed to open calibration file")?;
        let reader = BufReader::new(calibration_file);

        self.calibration =
            serde_json::from_reader(reader).context("Failed to parse calibration file")?;
        Ok(())
    }
}

pub struct MAX6675 {
    spi: Arc<Mutex<Spi>>,
    cs: OutputPin,
    pub id: usize,
}

impl MAX6675 {
    pub fn new(spi: Arc<Mutex<Spi>>, cs_pin: usize, id: usize) -> Self {
        let mut cs = Gpio::new()
            .expect("Failed to create new GPIO pin instance")
            .get(cs_pin as u8)
            .expect("Failed to get pin")
            .into_output();

        cs.set_high();

        Self { spi, cs, id }
    }

    pub fn read_temp(&mut self) -> Result<f64> {
        let mut spi = self.spi.lock().expect("BUG: Failed to acquire SPI device");

        // Custom CS implementation
        self.cs.set_low();
        let value = spi.read()?;
        self.cs.set_high();

        let mut value: usize = (value[0] as usize) << 8 | value[1] as usize;

        // If the sensor is mountend in wrong way, it can hold MISO high.
        if value == 0xFFFF_FFFF {
            return Err(anyhow!(
                "Sensor connection is probaly wrong, value 0x{value:08X}"
            ));
        }
        // If there are no data on the bus, there are still SCK present,
        // therefore 0x0 is read.
        if value == 0x0000_0000 {
            return Err(anyhow!(
                "Sensor is probaly not present, value 0x{value:08X}"
            ));
        }
        // Bit D2 indicates should be normally low.
        if value & 0x4 == 0x4 {
            return Err(anyhow!(
                "Thermocouple is probably not connected, value 0x{value:08X}"
            ));
        }

        // First 3 bits are just status flags
        value >>= 3;
        // Negative value, take 2's compliment. Compute this with subtraction.
        if value & 0x10000000 == 0x10000000 {
            value -= 4096
        }

        Ok(value as f64 * 0.25)
    }
}

pub async fn update_temp_periodically(
    descriptor: SensorDescriptor,
    temperatures: Arc<Mutex<Temperatures>>,
) {
    let spi = Arc::new(Mutex::new(Spi::open()));
    let mut sensors = Vec::new();
    for (id, cs_pin) in descriptor.cs_pins.iter().enumerate() {
        sensors.push(MAX6675::new(spi.clone(), *cs_pin, id));
    }

    loop {
        {
            let mut temperatures = temperatures
                .lock()
                .expect("BUG: Failed to acquire temperatures lock");

            temperatures.inner.clear();
            for sensor in sensors.iter_mut() {
                let temp = match sensor.read_temp() {
                    Ok(temp) => temp,
                    _ => {
                        warn!("Failed to read sensor_id {}", sensor.id);
                        continue;
                    }
                };

                temperatures.inner.insert(sensor.id, temp);

                if let Some(filtered_temperature) = temperatures.filtered.get_mut(&sensor.id) {
                    filtered_temperature.update(temp);
                }
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn calibrate_sensors(descriptor: SensorDescriptor, real_temp: f64) -> Result<()> {
    const NUM_MEASUREMENTS: usize = 180;
    // Minimal delay between measurements is 220 ms
    const MEAS_DELAY_MS: u64 = 330;

    let mut temperatures: BTreeMap<usize, Vec<f64>> = BTreeMap::new();

    let spi = Arc::new(Mutex::new(Spi::open()));
    let mut sensors = Vec::new();
    for (id, cs_pin) in descriptor.cs_pins.iter().enumerate() {
        sensors.push(MAX6675::new(spi.clone(), *cs_pin, id));
        temperatures.insert(id, Vec::new());
    }

    let meas_time_s = NUM_MEASUREMENTS * MEAS_DELAY_MS as usize / 1000;
    info!(
        "Acquiring temperatures, it will take {} seconds",
        meas_time_s
    );
    for _ in 0..NUM_MEASUREMENTS {
        for sensor in sensors.iter_mut() {
            if let Ok(temp) = sensor.read_temp() {
                let sensor_temps = temperatures
                    .get_mut(&sensor.id)
                    .expect("BUG: Failed to get sensor temperatures");
                sensor_temps.push(temp);
            } else {
                warn!("Failed to read temp from sensor {}", sensor.id);
            }
        }
        sleep(Duration::from_millis(MEAS_DELAY_MS)).await;
    }

    let mut calibration = BTreeMap::new();
    for (sensor_id, temps) in temperatures {
        let avg_temp: f64 = temps.iter().sum::<f64>() / temps.len() as f64;
        let offset = real_temp - avg_temp;
        debug!("sensor_id {sensor_id:2}, avg_temp {avg_temp:+6.2}, offset {offset:+5.2}");
        calibration.insert(sensor_id, offset);
    }
    store_calibration(calibration, &descriptor.calibration_file)
        .context("BUG: Failed to store calibration")?;
    Ok(())
}

pub fn store_calibration(calibration: BTreeMap<usize, f64>, path: &str) -> Result<()> {
    let mut calibration_file = File::create(path).context("Failed to open calibration file")?;
    calibration_file
        .write_all(
            serde_json::to_string_pretty(&calibration)
                .context("BUG: Failed to serialize calibration")?
                .as_bytes(),
        )
        .context("BUG: Failed to write calibration file")?;
    Ok(())
}
