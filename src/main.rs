mod max6675;
mod spi;

use log::debug;
use max6675::MAX6675;
use spi::Spi;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

fn main() {
    env_logger::init();

    const CS: [u8; 2] = [23, 24];

    let spi = Arc::new(Mutex::new(Spi::open()));
    let mut sensors = Vec::new();
    for (id, cs_pin) in CS.iter().enumerate() {
        sensors.push(MAX6675::new(spi.clone(), *cs_pin, id));
    }

    loop {
        for sensor in sensors.iter_mut() {
            let result = sensor.read_temp();
            debug!("meas result {:.2}", result.unwrap_or(f64::NAN));
        }
        thread::sleep(Duration::from_millis(1000));
    }
}
