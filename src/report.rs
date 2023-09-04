use std::io::Write;

use crate::configuration::RocketConfig;
use crate::report::latex::LatexElement;

mod latex;

const HEADER_CONTENT: &str = r#"\documentclass{article}

"#;

pub struct SensorReport;
// TODO: Calculation reports with a function based on variable names.
pub struct Report {
    config: RocketConfig
}

impl Report {
    pub fn new(config: RocketConfig) -> Report {
        Report { config }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{HEADER_CONTENT}")?;

        LatexElement::environment("document", vec![
            LatexElement::directive("section", vec![], vec!["Sensor Data".to_string()])
        ]).write(writer)
    }

    fn sensor_introduction(&self) -> String {
        let rocket_name = self.config.display_name();
        let sensor_count = self.config.sensors.len();
        let sensor_list = self.config.sensors.iter()
                                             .map(|s| s.name.as_str())
                                             .collect::<Vec<&str>>()
                                             .join(", ");
        format!("The {rocket_name} rocket has {sensor_count} sensors: {sensor_list}.")
        // TODO: Average data rate and things like that.
    }
}
