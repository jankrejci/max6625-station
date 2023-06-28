mod max6675;
mod spi;

use max6675::MAX6675;
use rocket::State;
use spi::Spi;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[macro_use]
extern crate rocket;

const NUM_SENSORS: usize = 12;
const CS_PINS: [u8; NUM_SENSORS] = [14, 4, 15, 18, 27, 23, 20, 5, 1, 7, 25, 24];

struct Temperatures {
    pub inner: HashMap<usize, f64>,
    pub calibration: HashMap<usize, f64>,
}

impl Temperatures {
    pub fn new(num_sensors: usize) -> Self {
        let mut default_calibration = HashMap::new();
        for sensor_id in 0..num_sensors {
            // Default calibration offset is 0.0 ˚C
            default_calibration.insert(sensor_id, 0.0);
        }

        Self {
            inner: HashMap::new(),
            calibration: default_calibration,
        }
    }
}

fn update_temp_periodically(temperatures: Arc<Mutex<Temperatures>>) {
    thread::spawn(move || {
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
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

#[get("/metrics")]
async fn metrics(temperatures: &State<Arc<Mutex<Temperatures>>>) -> String {
    let temperatures = temperatures
        .lock()
        .expect("BUG: Failed to acquire temperatures lock");

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("BUG: Failed to get current time")
        .as_millis();

    let mut metrics = String::new();
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
    let temperatures = Arc::new(Mutex::new(Temperatures::new(NUM_SENSORS)));

    update_temp_periodically(temperatures.clone());

    let _rocket = rocket::build()
        .mount("/", routes![metrics])
        .manage(temperatures.clone())
        .launch()
        .await?;

    Ok(())
}
