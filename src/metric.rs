use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Metric {
    value: Option<f64>,
    timestamp: u128,
    name: String,
    params: BTreeMap<String, String>,
}

impl Metric {
    pub fn new(name: &str, value: Option<f64>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("BUG: Failed to get current time")
            .as_millis();

        Self {
            value,
            timestamp,
            name: name.into(),
            params: BTreeMap::new(),
        }
    }
}
