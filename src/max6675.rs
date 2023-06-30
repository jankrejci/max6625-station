use crate::config::SensorDescriptor;
use crate::spi::Spi;
use anyhow::{anyhow, Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub struct Temperatures {
    pub inner: BTreeMap<usize, f64>,
    pub calibration: BTreeMap<usize, f64>,
}

impl Temperatures {
    pub fn new(num_sensors: usize) -> Self {
        let mut default_calibration = BTreeMap::new();
        for sensor_id in 0..num_sensors {
            // Default calibration offset is 0.0 ˚C
            default_calibration.insert(sensor_id, 0.0);
        }

        Self {
            inner: BTreeMap::new(),
            calibration: default_calibration,
        }
    }

    pub fn load_calibration(&mut self, path: &str) -> Result<()> {
        let calibration_file = File::open(path).context("Failed to read calibration file")?;
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
                if let Ok(temp) = sensor.read_temp() {
                    temperatures.inner.insert(sensor.id, temp);
                }
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}
