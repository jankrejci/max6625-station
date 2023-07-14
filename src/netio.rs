use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Clone, Deserialize, Debug)]
pub struct Descriptor {
    address: String,
    output: Vec<String>,
}

pub struct Netio {
    url: String,
}

impl Netio {
    pub fn new(descriptor: &Descriptor) -> Self {
        let url = format!("http://{}/netio.json", descriptor.address);
        Self { url }
    }

    pub async fn read_power(&self) -> Result<f64> {
        let response = reqwest::get(&self.url).await?.text().await?;
        let v: Value = serde_json::from_str(&response)?;

        let mut load = 0.0;
        load += v["Outputs"][2]["Load"]
            .as_f64()
            .ok_or(anyhow!("Failed to get load"))?;
        load += v["Outputs"][3]["Load"]
            .as_f64()
            .ok_or(anyhow!("Failed to get load"))?;

        Ok(load)
    }
}

pub async fn update_power_periodically(descriptor: Descriptor, power: Arc<Mutex<Option<f64>>>) {
    const UPDATE_PERIOD_MS: Duration = Duration::from_millis(500);

    let netio = Netio::new(&descriptor);

    loop {
        let power_reading = netio.read_power().await.ok();
        {
            let mut power = power.lock().expect("BUG: Failed to acquire voltagelock");
            *power = power_reading;
        }
        sleep(UPDATE_PERIOD_MS).await;
    }
}
