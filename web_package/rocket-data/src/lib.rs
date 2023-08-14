mod utils;

use std::io::BufReader;
use std::io::Write;

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

/// Given the location of a json config file and the binary data file, this
/// method will load the config file and parse the binary data file into
/// memory. The data will then be converted into CSV and written to a csv file.
///
/// # Arguments
///
/// * `config_file_location` - The location of the json config file.
/// * `data_file_location` - The location of the binary data file.
/// * `csv_file_location` - The location of the csv file to write to.
///
/// # Errors
///
/// This method will return an error if the config file cannot be loaded, the
/// data file cannot be opened, or the csv file cannot be written to.
///
/// # Returns
///
/// This method will return `Ok(())` if the csv file was successfully written
/// to, or an error if anything IO or parsing related went wrong.
#[wasm_bindgen]
pub fn convert_to_csv(config_file_location: &str, data_file_location: &str, csv_file_location: &str) -> Result<(), String> {
    let config: RocketConfig = flight_data_reader::load_config(config_file_location)?;

    let file = std::fs::File::open(data_file_location).map_err(|e| e.to_string())?;
    let file = BufReader::new(file);

    let packet_parser = PacketParser::new(file, config.clone());
    let mut csv_generator = CsvGenerator::new(packet_parser, config.clone());

    let mut output_csv_file = std::fs::File::create(csv_file_location).map_err(|e| e.to_string())?;
    while let Some(Ok(line)) = csv_generator.next() {
        output_csv_file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    }

    Ok(())
}
