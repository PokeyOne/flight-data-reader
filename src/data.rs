use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

use crate::configuration::{ValueKind, RocketConfig, ValueConfig};

#[cfg(test)]
mod tests;

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
    pub float_64: f64
}

impl Value {
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
            ValueKind::Float64 => self.float_64.to_string()
        }
    }
}

pub struct Packet {
    pub id: u8,
    pub values: Vec<Value>
}

pub struct PacketParser<R: Read> {
    reader: BufReader<R>,
    config: RocketConfig
}

impl<R: Read> PacketParser<R> {
    pub fn new(reader: R, config: RocketConfig) -> Self {
        Self {
            reader: BufReader::new(reader),
            config
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P, config: RocketConfig) -> std::io::Result<PacketParser<File>> {
        let file = File::open(path)?;
        Ok(PacketParser::new(file, config))
    }

    fn read_value(&mut self, value_config: &ValueConfig) -> std::io::Result<Value>  {
        match value_config.data_type {
            ValueKind::Int8 => {
                let mut value = [0u8; 1];
                self.reader.read_exact(&mut value)?;
                let value = i8::from_be_bytes(value);
                Ok(Value { int_8: value })
            },
            ValueKind::Int16 => {
                let mut value = [0u8; 2];
                self.reader.read_exact(&mut value)?;
                let value = i16::from_be_bytes(value);
                Ok(Value { int_16: value })
            },
            ValueKind::Int32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = i32::from_be_bytes(value);
                Ok(Value { int_32: value })
            },
            ValueKind::Int64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = i64::from_be_bytes(value);
                Ok(Value { int_64: value })
            },
            ValueKind::UInt8 => {
                let mut value = [0u8; 1];
                self.reader.read_exact(&mut value)?;
                let value = value[0];
                Ok(Value { uint_8: value })
            },
            ValueKind::UInt16 => {
                let mut value = [0u8; 2];
                self.reader.read_exact(&mut value)?;
                let value = u16::from_be_bytes(value);
                Ok(Value { uint_16: value })
            },
            ValueKind::UInt32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = u32::from_be_bytes(value);
                Ok(Value { uint_32: value })
            },
            ValueKind::UInt64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = u64::from_be_bytes(value);
                Ok(Value { uint_64: value })
            },
            ValueKind::Float32 => {
                let mut value = [0u8; 4];
                self.reader.read_exact(&mut value)?;
                let value = f32::from_be_bytes(value);
                Ok(Value { float_32: value })
            },
            ValueKind::Float64 => {
                let mut value = [0u8; 8];
                self.reader.read_exact(&mut value)?;
                let value = f64::from_be_bytes(value);
                Ok(Value { float_64: value })
            }
        }
    }
}

#[derive(Debug)]
pub enum PacketError {
    InvalidId(u8),
    InvalidValueCount {
        actual: usize,
        expected: usize
    }
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::InvalidId(id) => write!(f, "Invalid packet id: {}", id),
            PacketError::InvalidValueCount { actual, expected } => write!(f, "Invalid value count: expected {}, got {}", expected, actual)
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

        let mut values: Vec<Value> = Vec::with_capacity(sensor_config.values.len());

        let value_configs = sensor_config.clone().values;
        for value_config in value_configs {
            let value = self.read_value(&value_config).unwrap();
            values.push(value);
        }

        Some(Ok(Packet {
            id,
            values
        }))
    }
}
