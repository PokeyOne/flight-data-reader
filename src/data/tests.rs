use super::*;

const raw_config: &str = include_str!("../../example_config.json");

#[test]
fn test_packet_parser() {
    let config: RocketConfig = serde_json::from_str(raw_config).unwrap();
    let bin: Vec<u8> = vec![
        // LSM Sensor ID
        0x01,
        // x = 1.0
        0x3f, 0x80, 0x00, 0x00,
        // y = 1.0
        0x3f, 0x80, 0x00, 0x00,
        // z = 1.0
        0x3f, 0x80, 0x00, 0x00,
        // BMP Sensor ID
        0x02,
        // pressure = 1.0
        0x3f, 0x80, 0x00, 0x00,
        // temperature = 1.0
        0x3f, 0x80, 0x00, 0x00
    ];

    let mut packet_parser = PacketParser::new(bin.as_slice(), config);

    let packet = packet_parser.next().unwrap().unwrap();
    assert_eq!(packet.id, 1);
    assert_eq!(packet.values.len(), 3);
    for value in packet.values {
        assert_eq!(unsafe { value.float_32 }, 1.0);
    }

    let packet = packet_parser.next().unwrap().unwrap();
    assert_eq!(packet.id, 2);
    assert_eq!(packet.values.len(), 2);
    for value in packet.values {
        assert_eq!(unsafe { value.float_32 }, 1.0);
    }
}