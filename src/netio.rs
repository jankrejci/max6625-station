use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
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
        let response = reqwest::get(&self.url).await?;
        dbg!(response);
        Ok(0.0)
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
