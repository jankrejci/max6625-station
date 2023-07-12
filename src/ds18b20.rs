use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

struct DS18B20 {
    file: File,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Descriptor {
    id: String,
}

impl DS18B20 {
    const BASE_PATH: &str = "/sys/bus/w1/devices/";
    const TARGET: &str = "w1_slave";
    const UPDATE_PERIOD_MS: Duration = Duration::from_millis(400);

    pub fn open(id: &str) -> Self {
        let path = Path::new(Self::BASE_PATH).join(id).join(Self::TARGET);
        let file = File::open(path).expect("BUG: Failed to open sensor");

        Self { file }
    }

    pub fn read_temp(&mut self) -> Result<f64> {
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;
        let lines = buffer.split("\n");
        Ok(0.0)
    }

    pub async fn update_temp_periodically(
        descriptor: Descriptor,
        ambient_temperature: Arc<Mutex<Option<f64>>>,
    ) {
        let mut temp = Self::open(&descriptor.id);

        loop {
            let temperature_reading = temp.read_temp().ok();
            {
                let mut temperature = ambient_temperature
                    .lock()
                    .expect("BUG: Failed to acquire ambient_temperature lock");
                *temperature = temperature_reading;
            }
            sleep(Self::UPDATE_PERIOD_MS).await;
        }
    }
}
