use std::io::BufReader;

use crate::configuration::RocketConfig;

pub mod configuration;
pub mod csv;
pub mod data;
#[cfg(feature = "report")]
pub mod report;
pub mod result_table;

pub fn load_config(path: &str) -> Result<RocketConfig, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let file_reader = BufReader::new(file);

    serde_json::from_reader(file_reader).map_err(|e| e.to_string())
}

pub fn load_config_str(config: &str) -> Result<RocketConfig, String> {
    serde_json::from_str(config).map_err(|e| e.to_string())
}