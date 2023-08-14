use serde::Deserializer;
use serde_json::json;

use super::*;

#[test]
fn test_validate_with_conflicting_ids() {
    let config = RocketConfig {
        name: "test".to_string(),
        sensors: vec![
            SensorConfig {
                name: "sensor_a".to_string(),
                id: 4,
                values: vec![]
            },
            SensorConfig {
                name: "sensor_b".to_string(),
                id: 4,
                values: vec![]
            }
        ]
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_validate_normal() {
    let config = RocketConfig {
        name: "test".to_string(),
        sensors: vec![
            SensorConfig {
                name: "sensor_a".to_string(),
                id: 4,
                values: vec![]
            },
            SensorConfig {
                name: "sensor_b".to_string(),
                id: 7,
                values: vec![]
            }
        ]
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_sensor_by_id() {
    let config: RocketConfig = serde_json::from_value(json!({
        "name": "test",
        "sensors": [
            {
                "name": "sensor_a",
                "id": 4,
                "values": []
            },
            {
                "name": "sensor_b",
                "id": 7,
                "values": []
            }
        ]
    })).unwrap();

    assert_eq!(config.get_sensor_by_id(4).unwrap().name, "sensor_a");
    assert_eq!(config.get_sensor_by_id(7).unwrap().name, "sensor_b");
    assert!(config.get_sensor_by_id(8).is_none());
}

#[test]
fn test_load_from_json() {
    let file_content = include_str!("../../example_config.json");

    let config = serde_json::from_str::<RocketConfig>(file_content).unwrap();

    assert_eq!(config.name, "Xenia-2");
    assert_eq!(config.sensors.len(), 3);
    assert_eq!(config.get_sensor_by_id(1).unwrap().name, "LSM");
}