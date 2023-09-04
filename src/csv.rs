use std::error::Error;
use std::io::Write;

use std::fmt::Write as FmtWrite;

use crate::configuration::RocketConfig;
use crate::data::PacketError;

use crate::result_table::{SourceIterator, TableGenerator};

/// Iterator that generates CSV rows from data provided.
///
/// This uses the TableGenerator to generate the rows of a table and then
/// converts them to CSV rows. This iterator will also return the table header
/// as the first row. Each row will be a string containing the values separated
/// by commas, or the header if it's the first row.
pub struct CsvGenerator<I: SourceIterator> {
    is_first: bool,
    iter: TableGenerator<I>,
}

impl<I: SourceIterator> CsvGenerator<I> {
    /// Create a new CSV generator given a Packet iterator and a rocket
    /// configuration.
    ///
    /// This constructs the table generator so it can be used during the rest
    /// of the lifetime of the CSV generator.
    pub fn new(iter: I, config: RocketConfig) -> Self {
        Self {
            iter: TableGenerator::new(iter, config),
            is_first: true,
        }
    }

    /// Consume the iterator and write the CSV to the given writer.
    ///
    /// # Params
    ///
    /// * `writer` - Writer to write the CSV to.
    ///
    /// # Returns
    ///
    /// Result containing either nothing or an error.
    ///
    /// # Errors
    ///
    /// The two cases for error are either an IO error if the writer fails to
    /// write or a packet error if the iterator fails to yield a packet.
    pub fn write_csv<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        for row in self {
            let row = row?;
            writeln!(writer, "{}", row)?;
        }

        Ok(())
    }
}

impl<I: SourceIterator> Iterator for CsvGenerator<I> {
    type Item = Result<String, PacketError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Create the header row if this is the first row.
        if self.is_first {
            self.is_first = false;
            return Some(Ok(self.iter.column_names().join(",")));
        }

        // Get the next row from the table generator.
        let row = match self.iter.next()? {
            Ok(value) => value,
            Err(e) => return Some(Err(e)),
        };

        // Don't return anything if the row is empty.
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
    use crate::configuration::{ValueKind, SensorConfig, ValueConfig};
    use crate::data::{Value, Packet};

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
