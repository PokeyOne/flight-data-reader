use super::*;

use crate::configuration::ValueKind;
use crate::data::Value;

fn test_config() -> RocketConfig {
    RocketConfig {
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
    }
}

fn test_packets() -> Vec<Packet> {
    vec![
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
    ]
}

#[test]
fn test_basic_column_generation() {
    let mut table = TableGenerator::new(test_packets().into_iter().map(Ok), test_config().clone());

    assert_eq!(table.column_names(), vec!["test_value", "test_value2"]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(1.0_f32.into()), Some(1_i32.into())]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(2.0_f32.into()), Some(2_i32.into())]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(3.0_f32.into()), Some(3_i32.into())]);
    assert_eq!(table.next().is_none(), true);
}

#[test]
fn test_column_restriction() {
    let mut table = TableGenerator::new(test_packets().into_iter().map(Ok), test_config().clone());

    table.allow_columns(vec!["test_value2".to_string()]);

    assert_eq!(table.column_names(), vec!["test_value2"]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(1_i32.into())]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(2_i32.into())]);
    assert_eq!(table.next().unwrap().unwrap(), vec![Some(3_i32.into())]);
    assert_eq!(table.next().is_none(), true);
}