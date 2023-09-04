use std::collections::HashMap;

use crate::configuration::{RocketConfig, SensorConfig, ValueConfig};
use crate::data::{Packet, PacketError, TypedValue};

pub trait SourceIterator: Iterator<Item = Result<Packet, PacketError>> {}
impl<I: Iterator<Item = Result<Packet, PacketError>>> SourceIterator for I {}

/// Iterator that generates table rows from data provided.
///
/// The resulting rows will always have the values in the same order and the
/// order of column names can be retrieved from the
/// [`TableGenerator::column_names`].
pub struct TableGenerator<I: SourceIterator> {
    iter: I,
    config: RocketConfig,
    packet_buf: Vec<Packet>,
    columns: Vec<String>,
}

impl<I: SourceIterator> TableGenerator<I> {
    /// Create a new table generator given a Packet iterator and a rocket
    /// configuration.
    ///
    /// # Params
    ///
    /// * `iter` - Iterator that yields packets.
    /// * `config` - Rocket configuration, must be the same as the one used to
    ///              generate the packets.
    ///
    /// # Returns
    ///
    /// A new table generator.
    pub fn new(iter: I, config: RocketConfig) -> Self {
        let columns = Self::columns(&config);

        Self {
            iter,
            config,
            columns,
            packet_buf: vec![],
        }
    }

    /// Get a column name for a given sensor and value configuration.
    ///
    /// This is a static method mostly used during the construction of the
    /// generator in the [`TableGenerator::columns`] method.
    ///
    /// # Params
    ///
    /// * `sensor` - Sensor configuration.
    /// * `value` - Value configuration.
    ///
    /// # Returns
    ///
    /// A string containing the column name.
    pub fn column_name(sensor: &SensorConfig, value: &ValueConfig) -> String {
        format!("{}_{}", sensor.name, value.name)
    }

    /// Get a list of column names for a given rocket configuration.
    ///
    /// This is a static method mostly used during the construction of the
    /// generator.
    ///
    /// # Params
    ///
    /// * `config` - Rocket configuration.
    ///
    /// # Returns
    ///
    /// A vector containing the column names.
    pub fn columns(config: &RocketConfig) -> Vec<String> {
        let mut result = vec![];

        for sensor in config.sensors.iter() {
            for value in sensor.values.iter() {
                result.push(Self::column_name(sensor, value));
            }
        }

        result
    }

    /// An instance method of the [`TableGenerator::columns`] method to get the
    /// column names that are generated upon construction.
    pub fn column_names(&self) -> Vec<String> {
        self.columns.clone()
    }

    /// Get the next packet from the internal buffer or the source iterator if
    /// the buffer is empty.
    ///
    /// This method must be used instead of calling `next` on the source
    /// iterator directly. This is because the generator occasionally needs to
    /// buffer packets to ensure that the resulting table is consistent. This
    /// method will use any buffered packets before getting the next packet
    /// from the source iterator.
    ///
    /// # Returns
    ///
    /// The next packet from the source iterator or `None` if the source
    /// iterator is empty.
    fn next_packet(&mut self) -> Option<Result<Packet, PacketError>> {
        if self.packet_buf.is_empty() {
            self.iter.next()
        } else {
            Some(Ok(self.packet_buf.remove(0)))
        }
    }
}

impl<I: SourceIterator> Iterator for TableGenerator<I> {
    type Item = Result<Vec<Option<TypedValue>>, PacketError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_row: HashMap<String, TypedValue> = HashMap::new();

        'packet_loop: while let Some(packet) = self.next_packet() {
            // Propagate any errors from the source iterator.
            let packet = match packet {
                Ok(packet) => packet,
                Err(err) => return Some(Err(err)),
            };

            // Get the sensor configuration for this packet.
            let Some(sensor) = self.config.get_sensor_by_id(packet.id) else {
                return Some(Err(PacketError::InvalidId(packet.id)));
            };

            // If the number of values in the packet does not match the number
            // of values in the sensor configuration, we return an error.
            //
            // This should only really happen if the rocket configuration does
            // not match the one used for the packets.
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

            // Push all the values into the hashmap with the column names.
            for (spec, value) in sensor.values.iter().zip(packet.values.iter()) {
                let typed_value = unsafe { TypedValue::new(*value, &spec.data_type) };

                current_row.insert(Self::column_name(sensor, spec), typed_value);
            }
        }

        // Don't return empty rows.
        if current_row.is_empty() {
            return None;
        }

        // Combine all the values into a vector instead of a hashmap.
        let mut result = vec![];

        for column in &self.columns {
            match current_row.get(column) {
                Some(value) => result.push(Some(*value)),
                None => result.push(None),
            }
        }

        Some(Ok(result))
    }
}
