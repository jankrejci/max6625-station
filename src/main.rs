mod max6675;
mod spi;

use max6675::MAX6675;
use rocket::State;
use spi::Spi;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

#[macro_use]
extern crate rocket;

const CS_PINS: [u8; 10] = [14, 15, 18, 27, 23, 20, 1, 7, 25, 24];

struct Temperatures {
    inner: Mutex<Vec<(usize, f64)>>,
}

fn update_temp_periodically(temperatures: Arc<Temperatures>) {
    thread::spawn(move || {
        let spi = Arc::new(Mutex::new(Spi::open()));
        let mut sensors = Vec::new();
        for (id, cs_pin) in CS_PINS.iter().enumerate() {
            sensors.push(MAX6675::new(spi.clone(), *cs_pin, id));
        }

        loop {
            {
                let mut temperatures = temperatures
                    .inner
                    .lock()
                    .expect("Failed to acquire temperatures lock");

                temperatures.clear();
                for sensor in sensors.iter_mut() {
                    if let Ok(temp) = sensor.read_temp() {
                        temperatures.push((sensor.id, temp))
                    }
                }
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

#[get("/metrics")]
async fn metrics(temperatures: &State<Arc<Temperatures>>) -> String {
    let temperatures = temperatures
        .inner
        .lock()
        .expect("Failed to acquire temperatures lock");

    let mut metrics = String::new();
    for (id, temp) in temperatures.iter() {
        metrics.push_str(&format!("sensor_id: {id:3}, temperature {temp:6.2}\n"));
    }
    metrics
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let temperatures = Arc::new(Temperatures {
        inner: Mutex::new(vec![(0, 25.1), (99, 99.9)]),
    });

    update_temp_periodically(temperatures.clone());

    let _rocket = rocket::build()
        .mount("/", routes![metrics])
        .manage(temperatures.clone())
        .launch()
        .await?;

    Ok(())
}
