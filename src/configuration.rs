use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The type of a single scalar value.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    #[serde(rename = "int_8")]
    Int8,
    #[serde(rename = "int_16")]
    Int16,
    #[serde(rename = "int_32")]
    Int32,
    #[serde(rename = "int_64")]
    Int64,
    #[serde(rename = "uint_8")]
    UInt8,
    #[serde(rename = "uint_16")]
    UInt16,
    #[serde(rename = "uint_32")]
    UInt32,
    #[serde(rename = "uint_64")]
    UInt64,
    #[serde(rename = "float_32")]
    Float32,
    #[serde(rename = "float_64")]
    Float64,
}

impl ToString for ValueKind {
    fn to_string(&self) -> String {
        match self {
            ValueKind::Int8 => "int_8",
            ValueKind::Int16 => "int_16",
            ValueKind::Int32 => "int_32",
            ValueKind::Int64 => "int_64",
            ValueKind::UInt8 => "uint_8",
            ValueKind::UInt16 => "uint_16",
            ValueKind::UInt32 => "uint_32",
            ValueKind::UInt64 => "uint_64",
            ValueKind::Float32 => "float_32",
            ValueKind::Float64 => "float_64",
        }
        .to_string()
    }
}

/// A single value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueConfig {
    /// The name of the value.
    pub name: String,
    /// The kind of data that is read and stored.
    ///
    /// This is important as different kinds of value may have a different
    /// number of bytes.
    pub data_type: ValueKind,
}

/// Configuration for data from a sensor.
///
/// A sensor in the sense of this file format is simply a collection of values
/// that are read at the same time.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorConfig {
    /// The name of the sensor.
    pub name: String,
    /// The ID of this sensor in the rocket config.
    pub id: u8,
    /// The values that this sensor reads.
    pub values: Vec<ValueConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Endianess {
    Little,
    Big
}

impl Endianess {
    pub fn is_big(&self) -> bool {
        matches!(self, Endianess::Big)
    }
}

impl Default for Endianess {
    fn default() -> Self {
        Self::Big
    }
}

/// Configuration for a single rocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RocketConfig {
    /// The name of the rocket that is launching
    pub name: String,
    /// The sensors that are on the rocket
    pub sensors: Vec<SensorConfig>,
    #[serde(default = "Endianess::default")]
    pub endianess: Endianess
}

impl RocketConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids: HashSet<u8> = HashSet::new();

        for sensor in self.sensors.iter() {
            if ids.contains(&sensor.id) {
                return Err(format!("Multiple sensors with ID: {}", sensor.id));
            } else {
                ids.insert(sensor.id);
            }
        }

        Ok(())
    }

    pub fn get_sensor_by_id(&self, id: u8) -> Option<&SensorConfig> {
        // TODO: A way to keep this sorted would be handy.
        self.sensors.iter().find(|&sensor| sensor.id == id)
    }
}
