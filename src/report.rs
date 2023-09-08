use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::configuration::RocketConfig;
use crate::data::{PacketParser, TypedValue};
use crate::report::latex::LatexElement;
use crate::result_table::TableGenerator;

mod latex;

const HEADER_CONTENT: &str = r#"\documentclass{article}

"#;

/// Statistics on a value.
///
/// This is used during the construction of a report to calculate statistics on
/// the data.
#[derive(Debug, Clone)]
pub struct ValueStats {
    /// The minimum value recorded.
    min: TypedValue,
    /// The maximum value recorded.
    max: TypedValue,
    /// The number of samples recorded.
    count: u64,
}

/// Reported data about a sensor.
pub struct SensorReport {
    value_stats: HashMap<String, ValueStats>,
}

// TODO: Calculation reports with a function based on variable names.
pub struct Report {
    config: RocketConfig,
    sensor_reports: HashMap<u8, SensorReport>,
}

impl Report {
    pub fn new<R: Read>(config: RocketConfig, packet_parser: PacketParser<R>) -> Report {
        let mut table_generator = TableGenerator::new(packet_parser, config.clone());
        let mut sensor_reports = HashMap::new();

        let column_names = table_generator.column_names();
        let mut column_stats = HashMap::new();

        while let Some(row) = table_generator.next() {
            // TODO: Deal with errors.
            let row = row.unwrap();

            for (column_name, value) in column_names.iter().zip(row.iter()) {
                let Some(value) = value else {
                    continue;
                };

                let stats = column_stats
                    .entry(column_name.clone())
                    .or_insert_with(|| ValueStats {
                        min: value.clone(),
                        max: value.clone(),
                        count: 0,
                    });

                // If any of these fails, it means that the data is invalid.
                stats.min = stats.min.partial_min(value).unwrap();
                stats.max = stats.max.partial_max(value).unwrap();
                stats.count += 1;
            }
        }

        for sensor in config.sensors.iter() {
            let mut value_stats: HashMap<String, ValueStats> = HashMap::new();

            for value in sensor.values.iter() {
                let name = TableGenerator::<PacketParser<File>>::column_name(sensor, value);
                let Some(stats) = column_stats.get(&name) else {
                    continue;
                };
                value_stats.insert(value.name.clone(), stats.clone());
            }

            sensor_reports.insert(sensor.id, SensorReport { value_stats });
        }

        Report {
            config,
            sensor_reports,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{HEADER_CONTENT}")?;

        let mut elements = vec![
            LatexElement::Section("Sensor Data".to_string()),
            LatexElement::Raw(self.sensor_introduction()),
        ];

        for sensor in self.config.sensors.iter() {
            elements.push(LatexElement::Subsection(sensor.name.clone()));
            let value_list = sensor
                .values
                .iter()
                .map(|v| v.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ");
            elements.push(LatexElement::raw(format!(
                "The {} sensor has {} values: {}. ",
                sensor.name,
                sensor.values.len(),
                value_list
            )));

            let Some(sensor_report) = self.sensor_reports.get(&sensor.id) else {
                elements.push(LatexElement::raw("No data was recorded for this sensor. "));
                continue;
            };

            for (name, stats) in sensor_report.value_stats.iter() {
                elements.push(LatexElement::raw(format!(
                    "The {} value has {} samples. ",
                    name, stats.count
                )));
                elements.push(LatexElement::raw(format!(
                    "The minimum value is {}. ",
                    stats.min
                )));
                elements.push(LatexElement::raw(format!(
                    "The maximum value is {}. ",
                    stats.max
                )));
            }
        }

        LatexElement::environment("document", elements).write(writer)
    }

    fn sensor_introduction(&self) -> String {
        let rocket_name = self.config.display_name();
        let sensor_count = self.config.sensors.len();
        let sensor_list = self
            .config
            .sensors
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        format!("The {rocket_name} rocket has {sensor_count} sensors: {sensor_list}.")
        // TODO: Average data rate and things like that.
    }
}
