use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub scope: ScopeDescriptor,
    pub sensors: SensorDescriptor,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ScopeDescriptor {
    pub address: String,
    pub port: usize,
}

impl ScopeDescriptor {
    pub fn resource(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct SensorDescriptor {
    pub num_sensors: usize,
    pub cs_pins: Vec<usize>,
    pub calibration_file: String,
}

impl Config {
    pub fn load(path: &str) -> Self {
        let mut config_file = File::open(path).expect("BUG: Failed to open configuration file");
        let mut buffer = String::new();
        config_file
            .read_to_string(&mut buffer)
            .expect("BUG: Failed to read config file");

        toml::from_str(&buffer).expect("Failed to parse configuration file")
    }
}
