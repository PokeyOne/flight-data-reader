use std::collections::HashMap;
use std::io::{BufWriter, Write};

use std::fmt::Write as FmtWrite;

use crate::configuration::{RocketConfig, SensorConfig, ValueConfig};
use crate::data::{Packet, PacketError};

use crate::result_table::{SourceIterator, TableGenerator};

pub struct CsvGenerator<I: SourceIterator> {
    is_first: bool,
    iter: TableGenerator<I>
}

impl<I: SourceIterator> CsvGenerator<I> {
    pub fn new(iter: I, config: RocketConfig) -> Self {
        Self {
            iter: TableGenerator::new(iter, config),
            is_first: true
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
            return Some(Ok(self.iter.column_names().join(",")));
        }

        let row = match self.iter.next()? {
            Ok(value) => value,
            Err(e) => return Some(Err(e))
        };

        if row.is_empty() {
            return None;
        }

        // TODO: This might be better as iter to string and join.
        let mut result = String::new();

        for value in row {
            match value {
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
            endianess: crate::configuration::Endianess::default(),
            display_name: None,
            description: None,
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
