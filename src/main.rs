use std::path::{PathBuf, Path};

use clap::{Parser, Subcommand};
use flight_data_reader::configuration::RocketConfig;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
    /// The location of the config file.
    #[clap(short, long)]
    config: PathBuf
}

#[derive(Subcommand)]
enum Action {
    Check
}

fn main() {
    let args = Cli::parse();

    let config = match load_config(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Could not load config: {e}");
            return;
        }
    };

    match args.action {
        Action::Check => check_config(config)
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
