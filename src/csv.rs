use std::collections::HashMap;
use std::io::{BufWriter, Write};

use std::fmt::Write as FmtWrite;

use crate::configuration::{RocketConfig, SensorConfig, ValueConfig};
use crate::data::{Packet, PacketError};

pub trait SourceIterator: Iterator<Item = Result<Packet, PacketError>> {}
impl<I: Iterator<Item = Result<Packet, PacketError>>> SourceIterator for I {}

pub struct CsvGenerator<I: SourceIterator> {
    iter: I,
    is_first: bool,
    config: RocketConfig,
    packet_buf: Vec<Packet>,
    columns: Vec<String>,
}

impl<I: SourceIterator> CsvGenerator<I> {
    pub fn new(iter: I, config: RocketConfig) -> Self {
        let columns = Self::columns(&config);

        Self {
            iter,
            config,
            columns,
            is_first: true,
            packet_buf: vec![],
        }
    }

    pub fn column_name(sensor: &SensorConfig, value: &ValueConfig) -> String {
        format!("{}_{}", sensor.name, value.name)
    }

    pub fn columns(config: &RocketConfig) -> Vec<String> {
        let mut result = vec![];

        for sensor in config.sensors.iter() {
            for value in sensor.values.iter() {
                result.push(Self::column_name(sensor, value));
            }
        }

        result
    }

    fn next_packet(&mut self) -> Option<Result<Packet, PacketError>> {
        if self.packet_buf.is_empty() {
            self.iter.next()
        } else {
            Some(Ok(self.packet_buf.remove(0)))
        }
    }

    pub fn write_csv<W: Write>(&mut self, _writer: BufWriter<W>) {
        todo!()
    }
}

impl<I: SourceIterator> Iterator for CsvGenerator<I> {
    type Item = Result<String, PacketError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_first {
            self.is_first = false;
            return Some(Ok(self.columns.join(",")));
        }

        let mut current_row: HashMap<String, String> = HashMap::new();

        'packet_loop: while let Some(packet) = self.next_packet() {
            let packet = match packet {
                Ok(packet) => packet,
                Err(err) => return Some(Err(err)),
            };

            let Some(sensor) = self.config.get_sensor_by_id(packet.id) else {
                return Some(Err(PacketError::InvalidId(packet.id)));
            };

            if packet.values.len() != sensor.values.len() {
                return Some(Err(PacketError::InvalidValueCount {
                    expected: sensor.values.len(),
                    actual: packet.values.len(),
                }));
            }

            // If this packet has already been added to the current row, we
            // push the packet into a buffer and then end the row.
            for spec in sensor.values.iter() {
                if current_row.contains_key(&Self::column_name(sensor, spec)) {
                    self.packet_buf.push(packet);
                    break 'packet_loop;
                }
            }

            for (spec, value) in sensor.values.iter().zip(packet.values.iter()) {
                let value_str = unsafe { value.to_string(&spec.data_type) };

                current_row.insert(Self::column_name(sensor, spec), value_str);
            }
        }

        if current_row.is_empty() {
            return None;
        }

        let mut result = String::new();

        for column in &self.columns {
            match current_row.get(column) {
                Some(value) => write!(result, "{},", value).unwrap(),
                None => write!(result, ",").unwrap(),
            }
        }

        Some(Ok(result[0..result.len() - 1].to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::ValueKind;
    use crate::data::Value;

    use super::*;

    #[test]
    fn test_csv_generator() {
        let config = RocketConfig {
            name: "test".to_string(),
            sensors: vec![SensorConfig {
                id: 0,
                name: "test".to_string(),
                values: vec![
                    ValueConfig {
                        name: "value".to_string(),
                        data_type: ValueKind::Float32,
                    },
                    ValueConfig {
                        name: "value2".to_string(),
                        data_type: ValueKind::Int32,
                    },
                ],
            }],
        };

        let packets = vec![
            Packet {
                id: 0,
                values: vec![Value { float_32: 1.0 }, Value { int_32: 1 }],
            },
            Packet {
                id: 0,
                values: vec![Value { float_32: 2.0 }, Value { int_32: 2 }],
            },
            Packet {
                id: 0,
                values: vec![Value { float_32: 3.0 }, Value { int_32: 3 }],
            },
        ];

        let mut csv = CsvGenerator::new(packets.into_iter().map(Ok), config);

        assert_eq!(csv.next().unwrap().unwrap(), "test_value,test_value2");
        assert_eq!(csv.next().unwrap().unwrap(), "1.00000000,1");
        assert_eq!(csv.next().unwrap().unwrap(), "2.00000000,2");
        assert_eq!(csv.next().unwrap().unwrap(), "3.00000000,3");
        assert!(csv.next().is_none());
    }
}
