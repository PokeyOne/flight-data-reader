use std::collections::HashMap;

use crate::configuration::{RocketConfig, SensorConfig, ValueConfig};
use crate::data::{Packet, PacketError, TypedValue};

pub trait SourceIterator: Iterator<Item = Result<Packet, PacketError>> {}
impl<I: Iterator<Item = Result<Packet, PacketError>>> SourceIterator for I {}

pub struct TableGenerator<I: SourceIterator> {
    iter: I,
    config: RocketConfig,
    packet_buf: Vec<Packet>,
    columns: Vec<String>
}

impl<I: SourceIterator> TableGenerator<I> {
    pub fn new(iter: I, config: RocketConfig) -> Self {
        let columns = Self::columns(&config);

        Self {
            iter,
            config,
            columns,
            packet_buf: vec![]
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

    pub fn column_names(&self) -> Vec<String> {
        Self::columns(&self.config)
    }

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
                let typed_value = TypedValue::new(*value, &spec.data_type);

                current_row.insert(Self::column_name(sensor, spec), typed_value);
            }
        }

        if current_row.is_empty() {
            return None;
        }

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

