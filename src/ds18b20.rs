use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

struct DS18B20 {
    path: PathBuf,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Descriptor {
    id: String,
}

impl DS18B20 {
    const BASE_PATH: &str = "/sys/bus/w1/devices/";
    const TARGET: &str = "w1_slave";

    pub fn open(id: &str) -> Self {
        let path = Path::new(Self::BASE_PATH).join(id).join(Self::TARGET);
        Self { path }
    }

    pub fn read_temp(&self) -> Result<f64> {
        let buffer = fs::read_to_string(&self.path)?;
        let lines: Vec<_> = buffer.trim().split('\n').collect();

        let crc_check = lines.first().expect("BUG: Failed to get data");
        if !crc_check.contains("YES") {
            return Err(anyhow!("Failed to obtain valid data"));
        }

        let value = lines.last().expect("BUG: Failed to get data");
        let value: Vec<_> = value.split('=').collect();
        let value = value.last().expect("BUG: Failed to get temperature");
        let value = f64::from_str(value)?;

        Ok(value / 1000.0)
    }
}

pub async fn update_temp_periodically(
    descriptor: Descriptor,
    ambient_temperature: Arc<Mutex<Option<f64>>>,
) {
    const UPDATE_PERIOD_MS: Duration = Duration::from_millis(400);

    let temp = DS18B20::open(&descriptor.id);

    loop {
        let temperature_reading = temp.read_temp().ok();
        {
            let mut temperature = ambient_temperature
                .lock()
                .expect("BUG: Failed to acquire ambient_temperature lock");
            *temperature = temperature_reading;
        }
        sleep(UPDATE_PERIOD_MS).await;
    }
}
