use std::io::BufReader;

use crate::configuration::RocketConfig;

pub mod configuration;
pub mod csv;
pub mod data;
#[cfg(feature = "report")]
pub mod report;

pub fn load_config(path: &str) -> Result<RocketConfig, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let file_reader = BufReader::new(file);

    serde_json::from_reader(file_reader).map_err(|e| e.to_string())
}
