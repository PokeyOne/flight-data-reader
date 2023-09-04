use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::configuration::{RocketConfig, ValueConfig, ValueKind};

#[cfg(test)]
mod tests;

/// A safe wrapper around a value that knows its type.
#[derive(Clone, Copy)]
pub struct TypedValue {
    pub value: Value,
    pub value_kind: ValueKind,
}

impl TypedValue {
    pub fn new(value: Value, value_kind: &ValueKind) -> Self {
        Self {
            value,
            value_kind: *value_kind,
        }
    }
}

impl Display for TypedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.value.to_string(&self.value_kind) })
    }
}

/// A value that is recorded in a packet.
///
/// This value is a union and does not store the internal type. For user-facing
/// implementations that need to know the type, see the `TypedValue` struct.
#[derive(Clone, Copy)]
pub union Value {
    pub int_8: i8,
    pub int_16: i16,
    pub int_32: i32,
    pub int_64: i64,
    pub uint_8: u8,
    pub uint_16: u16,
    pub uint_32: u32,
    pub uint_64: u64,
    pub float_32: f32,
    pub float_64: f64,
}

impl Value {
    /// Converts the value to a string using the format method.
    ///
    /// Integers are simply converted to a string using the format! macro with
    /// no special formatting. Floats are formatted with 8 decimal places.
    ///
    /// # Arguments
    ///
    /// * `value_kind` - The kind of value to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// use flight_data_reader::data::Value;
    /// use flight_data_reader::configuration::ValueKind;
    ///
    /// let value_a = Value { int_8: 42 };
    /// let value_b = Value { float_32: 42.42 };
    ///
    /// assert_eq!(unsafe { value_a.to_string(&ValueKind::Int8) }, "42");
    /// assert_eq!(unsafe { value_b.to_string(&ValueKind::Float32) }, "42.41999817");
    /// ```
    ///
    /// # Safety
    ///
    /// This method is unsafe because it is possible to call it with the wrong
    /// value kind. For example, if the value is an int_8, but the value kind
    /// is a float_32, then the value will be interpreted as a float_32, which
    /// will result in either undefined behavior or a panic.
    pub unsafe fn to_string(&self, value_kind: &ValueKind) -> String {
        match value_kind {
            ValueKind::Int8 => self.int_8.to_string(),
            ValueKind::Int16 => self.int_16.to_string(),
            ValueKind::Int32 => self.int_32.to_string(),
            ValueKind::Int64 => self.int_64.to_string(),
            ValueKind::UInt8 => self.uint_8.to_string(),
            ValueKind::UInt16 => self.uint_16.to_string(),
            ValueKind::UInt32 => self.uint_32.to_string(),
            ValueKind::UInt64 => self.uint_64.to_string(),
            ValueKind::Float32 => format!("{:.8}", self.float_32),
            ValueKind::Float64 => format!("{:.8}", self.float_64),
        }
    }
}

/// A single reading of a sensor.
///
/// Each packet consists of a single byte indicating the sensor ID that is in
/// the config file, followed by the values for that sensor.
pub struct Packet {
    /// The ID of the sensor that is read.
    pub id: u8,
    /// The values that are read from that sensor.
    ///
    /// The config is required to know the types of the values and how to
    /// retrieve them.
    pub values: Vec<Value>,
}

/// A parser for reading packets from a stream.
///
/// This struct takes a reader that can be used as a byte stream of data, and
/// presents an iterator interface that can be used to read packets
/// individually.
pub struct PacketParser<R: Read> {
    /// The input reader.
    reader: BufReader<R>,
    /// The configuration used to know how many values and what kind of values
    /// to read.
    config: RocketConfig,
}

macro_rules! from_le_or_be_bytes {
    ($kind:ident, $value:expr, $endianess:expr) => {
        if $endianess.is_big() {
            $kind::from_be_bytes($value)
        } else {
            $kind::from_le_bytes($value)
        }
    };
}

impl<R: Read> PacketParser<R> {
    /// Create a new packet parser from a reader and a config.
    pub fn new(reader: R, config: RocketConfig) -> Self {
        Self {
            reader: BufReader::new(reader),
            config,
        }
    }

    /// Create a new packet parser from a path and a config.
    ///
    /// The file is opened and a packet parser is created from the file.
    ///
    /// # Returns
    ///
    /// This method will return a result with either a packet parser or an IO
    /// error if the file cannot be opened.
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        config: RocketConfig,
    ) -> std::io::Result<PacketParser<File>> {
        let file = File::open(path)?;
        Ok(PacketParser::new(file, config))
    }

    fn read_value(&mut self, value_config: &ValueConfig) -> std::io::Result<Value> {
        match value_config.data_type {
            ValueKind::Int8 => {
                let mut value = [0u8; 1];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(i8, value, self.config.endianess);
                Ok(Value { int_8: value })
            }
            ValueKind::Int16 => {
                let mut value = [0u8; 2];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(i16, value, self.config.endianess);
                Ok(Value { int_16: value })
            }
            ValueKind::Int32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(i32, value, self.config.endianess);
                Ok(Value { int_32: value })
            }
            ValueKind::Int64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(i64, value, self.config.endianess);
                Ok(Value { int_64: value })
            }
            ValueKind::UInt8 => {
                let mut value = [0u8; 1];
                self.reader.read_exact(&mut value)?;
                let value = value[0];
                Ok(Value { uint_8: value })
            }
            ValueKind::UInt16 => {
                let mut value = [0u8; 2];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(u16, value, self.config.endianess);
                Ok(Value { uint_16: value })
            }
            ValueKind::UInt32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(u32, value, self.config.endianess);
                Ok(Value { uint_32: value })
            }
            ValueKind::UInt64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(u64, value, self.config.endianess);
                Ok(Value { uint_64: value })
            }
            ValueKind::Float32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(f32, value, self.config.endianess);
                Ok(Value { float_32: value })
            }
            ValueKind::Float64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = from_le_or_be_bytes!(f64, value, self.config.endianess);
                Ok(Value { float_64: value })
            }
        }
    }
}

/// An error that occurs while parsing a packet.
#[derive(Debug)]
pub enum PacketError {
    /// The parser tried to read an ID, but that ID was not in the config.
    ///
    /// The contained value is the ID that was read.
    InvalidId(u8),
    /// Some how a parse value had the wrong number of values compared to the
    /// config.
    ///
    /// This is likely only to happen if the packets are loaded with a different
    /// config than the one used to later process the packets.
    InvalidValueCount { actual: usize, expected: usize },
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::InvalidId(id) => write!(f, "Invalid packet id: {}", id),
            PacketError::InvalidValueCount { actual, expected } => write!(
                f,
                "Invalid value count: expected {}, got {}",
                expected, actual
            ),
        }
    }
}

impl std::error::Error for PacketError {}

impl<R: Read> Iterator for PacketParser<R> {
    type Item = Result<Packet, PacketError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut id = [0u8; 1];

        if self.reader.read_exact(&mut id).is_err() {
            return None;
        }

        let id = id[0];

        let Some(sensor_config) = self.config.get_sensor_by_id(id) else {
            return Some(Err(PacketError::InvalidId(id)));
        };
        println!("Reading sensor {}", &sensor_config.name);

        let mut values: Vec<Value> = Vec::with_capacity(sensor_config.values.len());

        let value_configs = sensor_config.clone().values;
        for value_config in value_configs {
            let value = self.read_value(&value_config).unwrap();
            println!(
                "Read value: {} ({})",
                unsafe { value.to_string(&value_config.data_type) },
                &value_config.name
            );
            values.push(value);
        }

        Some(Ok(Packet { id, values }))
    }
}
