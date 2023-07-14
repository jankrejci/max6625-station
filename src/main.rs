mod args;
mod config;
mod ds18b20;
mod kalman;
mod max6675;
mod netio;
mod scope;
mod spi;

use anyhow::Result;
use clap::Parser;
use log::info;
use max6675::Temperatures;
use rocket::State;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[macro_use]
extern crate rocket;

struct Measurements {
    temperatures: Arc<Mutex<Temperatures>>,
    ambient_temperature: Arc<Mutex<Option<f64>>>,
    psu_voltage: Arc<Mutex<Option<f64>>>,
    fan_rpm: Arc<Mutex<Option<f64>>>,
    power: Arc<Mutex<Option<f64>>>,
}

#[get("/metrics")]
async fn metrics(measurements: &State<Measurements>) -> String {
    let mut metrics = String::new();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("BUG: Failed to get current time")
        .as_millis();

    if let Some(psu_voltage) = *measurements
        .psu_voltage
        .lock()
        .expect("BUG: Failed to acquire psu_voltage lock")
    {
        metrics.push_str(&format!("scope_voltage_v {psu_voltage:.2} {time}\n"));
    }

    if let Some(fan_rpm) = *measurements
        .fan_rpm
        .lock()
        .expect("BUG: Failed to acquire fan_rpm lock")
    {
        metrics.push_str(&format!("scope_fan_rpm {fan_rpm:.0} {time}\n"));
    }

    if let Some(ambient_temperature) = *measurements
        .ambient_temperature
        .lock()
        .expect("BUG: Failed to acquire ambient_temperature lock")
    {
        metrics.push_str(&format!(
            "ds18b20_temperature_c {ambient_temperature:.2} {time}\n"
        ));
    }

    let temperatures = measurements
        .temperatures
        .lock()
        .expect("BUG: Failed to acquire temperatures lock");

    for (sensor_id, temp) in temperatures.inner.iter() {
        if let Some(calibration_offset) = temperatures.calibration.get(sensor_id) {
            let temp = temp + calibration_offset;
            metrics.push_str(&format!(
                "max6675_temperature_c{{sensor_id=\"{sensor_id}\"}} {temp:.2} {time}\n"
            ));
        }
    }

    for (sensor_id, filtered) in temperatures.filtered.iter() {
        if let Some(calibration_offset) = temperatures.calibration.get(sensor_id) {
            let temp = filtered.value() + calibration_offset;
            metrics.push_str(&format!(
                "max6675_temperature_filtered_c{{sensor_id=\"{sensor_id}\"}} {temp:.2} {time}\n"
            ));
        }
    }

    for (sensor_id, temp) in temperatures.inner.iter() {
        metrics.push_str(&format!(
            "max6675_temperature_raw_c{{sensor_id=\"{sensor_id}\"}} {temp:.2} {time}\n"
        ));
    }

    metrics
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    env_logger::init();
    let args = args::Cli::parse();
    let config = config::Config::load(&args.config);

    if let Some(real_temp) = args.calibrate {
        info!("Calibrating sensors to temperature {} ˚C", real_temp);
        max6675::calibrate_sensors(config.sensors.clone(), real_temp, &config.kalman)
            .await
            .expect("BUG: Failed to calibrate sensors");
        return Ok(());
    }

    let mut temperatures = Temperatures::new(config.sensors.num_sensors, &config.kalman);
    temperatures
        .load_calibration(&config.sensors.calibration_file)
        .unwrap_or_else(|_| warn!("Failed to load calibration"));
    let temperatures = Arc::new(Mutex::new(temperatures));

    let ambient_temperature = Arc::new(Mutex::new(None));
    let psu_voltage = Arc::new(Mutex::new(None));
    let fan_rpm = Arc::new(Mutex::new(None));
    let power = Arc::new(Mutex::new(None));

    let measurements = Measurements {
        temperatures: temperatures.clone(),
        ambient_temperature: ambient_temperature.clone(),
        psu_voltage: psu_voltage.clone(),
        fan_rpm: fan_rpm.clone(),
        power: power.clone(),
    };

    tokio::spawn(max6675::update_temp_periodically(
        config.sensors.clone(),
        temperatures.clone(),
    ));
    tokio::spawn(ds18b20::update_temp_periodically(
        config.ds18b20.clone(),
        ambient_temperature.clone(),
    ));
    tokio::spawn(scope::update_voltage_periodically(
        config.scope.clone(),
        psu_voltage.clone(),
        fan_rpm.clone(),
    ));
    tokio::spawn(netio::update_power_periodically(
        config.netio.clone(),
        power.clone(),
    ));

    let _rocket = rocket::build()
        .mount("/", routes![metrics])
        .manage(measurements)
        .launch()
        .await?;

    Ok(())
}
