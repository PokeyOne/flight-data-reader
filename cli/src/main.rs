use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use flight_data_reader::configuration::RocketConfig;
use flight_data_reader::csv::CsvGenerator;
use flight_data_reader::data::PacketParser;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Check {
        /// The location of the config file.
        #[clap(short, long)]
        config: PathBuf,
    },
    Convert {
        #[clap(short, long, default_value = "csv")]
        to: String,
        /// The location of the config file.
        #[clap(short, long)]
        config: PathBuf,
        /// The encoded file from the flight computer.
        data: PathBuf,
        /// The location to write the decoded data to.
        output: PathBuf,
    },
}

impl Action {
    pub fn config(&self) -> &Path {
        match self {
            Action::Check { config } => config,
            Action::Convert { config, .. } => config,
        }
    }
}

fn main() {
    let args = Cli::parse();

    let config = match load_config(args.action.config()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Could not load config: {e}");
            return;
        }
    };

    match args.action {
        Action::Check { .. } => check_config(config),
        Action::Convert {
            to, data, output, ..
        } => convert_data(config, to, data, output),
    }
}

fn convert_data(config: RocketConfig, to: String, data: PathBuf, output: PathBuf) {
    let input_reader = File::open(data).unwrap();
    let packet_parser = PacketParser::new(input_reader, config.clone());
    let mut csv_gen = CsvGenerator::new(packet_parser, config.clone());

    let mut output_writer = BufWriter::new(File::create(output).unwrap());

    while let Some(line) = csv_gen.next() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Error while parsing packet: {e}");
                continue;
            }
        };

        output_writer.write_all(line.as_bytes()).unwrap();
        output_writer.write_all(b"\n").unwrap();
    }
}

fn check_config(config: RocketConfig) {
    println!("Loaded config:");
    println!("  name: {}", config.name);
    println!("  sensors: {}", config.sensors.len());
    println!();

    if let Err(msg) = config.validate() {
        println!("Rocket invalid: {msg}");
    } else {
        println!("Configuration is valid!");
    }

    println!();
    println!("Sensors:");

    for sensor in config.sensors.iter() {
        println!("  [\n    name: {},\n    values: [", sensor.name);
        for value in sensor.values.iter() {
            println!("      {}: {},", value.name, value.data_type.to_string());
        }
        println!("    ]");
        println!("  ]");
    }
}

fn load_config<P: AsRef<Path>>(path: P) -> Result<RocketConfig, Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(path)?;

    let config = serde_json::from_str(&file_content)?;

    Ok(config)
}
