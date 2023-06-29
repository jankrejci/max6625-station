mod max6675;
mod scope;
mod spi;

use anyhow::{Context, Result};
use max6675::MAX6675;
use rocket::State;
use scope::Scope;
use spi::Spi;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[macro_use]
extern crate rocket;

const NUM_SENSORS: usize = 12;
const CS_PINS: [u8; NUM_SENSORS] = [14, 4, 15, 18, 27, 23, 20, 5, 1, 7, 25, 24];
const CALIBRATION_FILE: &str = "calibration.json";
const SCOPE_RESOURCE: &str = "10.33.50.233:5025";

struct Measurements {
    temperatures: Arc<Mutex<Temperatures>>,
    voltage: Arc<Mutex<Option<f64>>>,
}

struct Temperatures {
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

async fn update_temp_periodically(temperatures: Arc<Mutex<Temperatures>>) {
    let spi = Arc::new(Mutex::new(Spi::open()));
    let mut sensors = Vec::new();
    for (id, cs_pin) in CS_PINS.iter().enumerate() {
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

async fn update_voltage_periodically(voltage: Arc<Mutex<Option<f64>>>) {
    let mut scope = Scope::open(SCOPE_RESOURCE).await;
    scope.init().await.expect("BUG: Failed to initialize scope");

    loop {
        let voltage_reading = scope.read_mean().await.ok();
        {
            let mut voltage = voltage.lock().expect("BUG: Failed to acquire voltagelock");
            *voltage = voltage_reading;
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

#[get("/metrics")]
async fn metrics(measurements: &State<Measurements>) -> String {
    let temperatures = measurements
        .temperatures
        .lock()
        .expect("BUG: Failed to acquire temperatures lock");

    let voltage = measurements
        .voltage
        .lock()
        .expect("BUG: Failed to acquire voltage lock");

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("BUG: Failed to get current time")
        .as_millis();

    let mut metrics = String::new();
    if let Some(voltage) = *voltage {
        metrics.push_str(&format!("scope_voltage_v {voltage:.2} {time}\n"));
    }
    for (sensor_id, temp) in temperatures.inner.iter() {
        if let Some(calibration_offset) = temperatures.calibration.get(sensor_id) {
            let temp = temp - calibration_offset;
            metrics.push_str(&format!(
                "max6675_temperature_c{{sensor_id=\"{sensor_id}\"}} {temp:.2} {time}\n"
            ));
        }
    }
    metrics
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let mut temperatures = Temperatures::new(NUM_SENSORS);
    temperatures
        .load_calibration(CALIBRATION_FILE)
        .expect("Failed to load calibration");
    let temperatures = Arc::new(Mutex::new(temperatures));

    let voltage = Arc::new(Mutex::new(None));

    let measurements = Measurements {
        temperatures: temperatures.clone(),
        voltage: voltage.clone(),
    };

    tokio::spawn(update_temp_periodically(temperatures.clone()));
    tokio::spawn(update_voltage_periodically(voltage.clone()));

    let _rocket = rocket::build()
        .mount("/", routes![metrics])
        .manage(measurements)
        .launch()
        .await?;

    Ok(())
}
