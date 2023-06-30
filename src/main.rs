mod args;
mod config;
mod max6675;
mod scope;
mod spi;

use anyhow::Result;
use clap::Parser;
use max6675::Temperatures;
use rocket::State;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[macro_use]
extern crate rocket;

struct Measurements {
    temperatures: Arc<Mutex<Temperatures>>,
    voltage: Arc<Mutex<Option<f64>>>,
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
    let args = args::Cli::parse();
    let config = config::Config::load(&args.config);

    if let Some(real_temp) = args.calibration {
        println!("Calibrating");
        return Ok(());
    }

    let mut temperatures = Temperatures::new(config.sensors.num_sensors);
    temperatures
        .load_calibration(&config.sensors.calibration_file)
        .expect("Failed to load calibration");
    let temperatures = Arc::new(Mutex::new(temperatures));

    let voltage = Arc::new(Mutex::new(None));

    let measurements = Measurements {
        temperatures: temperatures.clone(),
        voltage: voltage.clone(),
    };

    tokio::spawn(max6675::update_temp_periodically(
        config.sensors.clone(),
        temperatures.clone(),
    ));
    tokio::spawn(scope::update_voltage_periodically(
        config.scope.clone(),
        voltage.clone(),
    ));

    let _rocket = rocket::build()
        .mount("/", routes![metrics])
        .manage(measurements)
        .launch()
        .await?;

    Ok(())
}
