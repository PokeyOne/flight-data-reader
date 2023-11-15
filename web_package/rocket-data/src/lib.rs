mod utils;

use flight_data_reader::configuration::RocketConfig;
use flight_data_reader::csv::CsvGenerator;
use flight_data_reader::data::PacketParser;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, rocket-data!");
}

/// Given a config file and a binary data file, this method will
/// load the config file and parse the binary data file into memory.
/// The data will then be converted into CSV and returned as a String.
///
/// # Arguments
///
/// * `config_file` - The json config file JavaScript File object.
/// * `data_file_location` - The binary data file Javascript File object.
///
/// # Errors
///
/// This method will return an error if the config file cannot be loaded,
/// or the data file cannot be loaded.
///
/// # Returns
///
/// This method will return the stringified CSV file or
/// an error if file loading or parsing related went wrong.

#[wasm_bindgen]
pub async fn convert_to_csv(config_file: web_sys::File, data_file: web_sys::File) -> Result<String, String> {
    let config = wasm_bindgen_futures::JsFuture::from(config_file.text())
        .await
        .ok()
        .expect("Could not get rocket config data")
        .as_string()
        .unwrap();
    let data = wasm_bindgen_futures::JsFuture::from(data_file.text())
        .await
        .ok()
        .expect("Could not get binary data")
        .as_string()
        .unwrap();
    let data = data.as_bytes();
    
    let config: RocketConfig = flight_data_reader::load_config_str(&config)?;
    let packet_parser = PacketParser::new(data, config.clone());
    let mut csv_generator = CsvGenerator::new(packet_parser, config.clone());

    let mut csv_str = String::new();
    while let Some(Ok(line)) = csv_generator.next() {
	    csv_str.push_str(&line);
	    csv_str.push('\n');
    }

    Ok(csv_str)
}
