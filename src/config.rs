use crate::kalman;
use crate::max6675;
use crate::scope;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub scope: scope::Descriptor,
    pub sensors: max6675::Descriptor,
    pub kalman: kalman::Descriptor,
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
