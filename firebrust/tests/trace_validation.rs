// https://www.codertools.net/tools/crc.php
// Para el input format ejemplo: 01 07
// Para el CRC Result: E241, rotar a 0x41 0xEC

use firebrust::{ProcessedTraces, RawTraces, RegisterUnits, get_modbus_function_name};

#[test]
fn read_coils_validation() {
    // 01 01 00 19 00 03 AD CC
    let traces: Vec<u8> = vec![0x01, 0x01, 0x00, 0x19, 0x00, 0x03, 0xAD, 0xCC];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 1,
            function_name: get_modbus_function_name(1).into(),
            address_unit_1: Some(0x19),
            quantity_unit_1: Some(3),
            register_units: RegisterUnits::default(),
            crc_calculated: 0xCCAD,
            ..Default::default()
        })
    );
}

#[test]
fn read_discrete_inputs() {
    // 01 02 00 04 00 0C 39 CE
    let traces: Vec<u8> = vec![0x01, 0x02, 0x00, 0x04, 0x00, 0x0C, 0x39, 0xCE];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 2,
            function_name: get_modbus_function_name(2).into(),
            address_unit_1: Some(0x04),
            quantity_unit_1: Some(0xC),
            register_units: RegisterUnits::default(),
            crc_calculated: 0xCE39,
            ..Default::default()
        })
    );
}

#[test]
fn read_holding_registers() {
    // 01 03 04 EC 00 0C 85 0A
    let traces: Vec<u8> = vec![0x01, 0x03, 0x04, 0xEC, 0x00, 0x0C, 0x85, 0x0A];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 3,
            function_name: get_modbus_function_name(3).into(),
            address_unit_1: Some(0x04EC),
            quantity_unit_1: Some(0x000C),
            register_units: RegisterUnits::default(),
            crc_calculated: 0x0A85,
            ..Default::default()
        })
    );
}

// read_input_registers

#[test]
fn read_input_registers() {
    // 01 04 00 19 00 05 E1 CE
    let traces: Vec<u8> = vec![0x01, 0x04, 0x00, 0x19, 0x00, 0x05, 0xE1, 0xCE];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 4,
            function_name: get_modbus_function_name(4).into(),
            address_unit_1: Some(0x0019),
            quantity_unit_1: Some(0x0005),
            register_units: RegisterUnits::default(),
            crc_calculated: 0xCEE1,
            ..Default::default()
        })
    );
}

#[test]
fn write_single_coil() {
    // 01 05 00 OA FF 00 AC 38
    let traces: Vec<u8> = vec![0x01, 0x05, 0x00, 0x0A, 0xFF, 0x00, 0xAC, 0x38];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 5,
            function_name: get_modbus_function_name(5).into(),
            address_unit_1: Some(0x00A),
            quantity_unit_1: Some(0xFF00),
            crc_calculated: 0x38AC,
            ..Default::default()
        })
    );
}

#[test]
fn write_single_register() {
    // 01 06 04 FD 06 DD DA F3
    let traces: Vec<u8> = vec![0x01, 0x06, 0x04, 0xFD, 0x06, 0xDD, 0xDA, 0xF3];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 6,
            function_name: get_modbus_function_name(6).into(),
            address_unit_1: Some(0x04FD),
            register_units: RegisterUnits {
                mininum_value_register: Some(0x06DD),
                maximum_value_register: Some(0x06DD),
                median_value_register: Some(0x06DD),
                total_value_register: Some(0x06DD),
                zeros_count_register: Some(((0x06DD as i16).count_zeros()) as i64)
            },
            crc_calculated: 0xF3DA,
            ..Default::default()
        })
    );
}

#[test]
fn read_exception_status() {
    // 01 07 41 E2

    let traces: Vec<u8> = vec![0x01, 0x07, 0x41, 0xE2];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 7,
            function_name: get_modbus_function_name(7).into(),
            crc_calculated: 0xE241,
            ..Default::default()
        })
    );
}

#[test]
fn diagnostics() {
    // 01 08 00 00 12 34 ED 7C

    let traces: Vec<u8> = vec![0x01, 0x08, 0x00, 0x00, 0x12, 0x34, 0xED, 0x7C];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 8,
            function_name: get_modbus_function_name(8).into(),
            crc_calculated: 0x7CED,
            ..Default::default()
        })
    );
}

#[test]
fn get_com_event_counter() {
    // 01 0B 04 0A

    let traces: Vec<u8> = vec![0x01, 0x0B, 0x41, 0xE7];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 11,
            function_name: get_modbus_function_name(11).into(),
            crc_calculated: 0xE741,
            ..Default::default()
        })
    );
}

#[test]
fn get_com_event_log() {
    // 01 0C 00 25

    let traces: Vec<u8> = vec![0x01, 0x0B, 0x41, 0xE7];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 11,
            function_name: get_modbus_function_name(11).into(),
            crc_calculated: 0xE741,
            ..Default::default()
        })
    );
}

#[test]
fn write_multiple_coils_01() {
    // 01 OF 00 OA 00 OC 02 FF 07 E5 28

    let traces: Vec<u8> = vec![
        0x01, 0x0F, 0x00, 0x0A, 0x00, 0x0C, 0x02, 0xFF, 0x07, 0xE5, 0x28,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 15,
            function_name: get_modbus_function_name(15).into(),
            address_unit_1: Some(10),
            quantity_unit_1: Some(12),
            count_unit_1: Some(2),
            register_units: RegisterUnits {
                mininum_value_register: Some(7),
                maximum_value_register: Some(255),
                median_value_register: Some(131),
                total_value_register: Some(255 + 7),
                zeros_count_register: Some(
                    ((7 as u16).count_zeros() + (255 as u16).count_zeros()) as i64
                )
            },
            crc_calculated: 0x28E5,
            ..Default::default()
        })
    );
}

#[test]
fn write_multiple_coils_02() {
    // 01 OF 00 OA 00 14 03 FF 40 OF BB 81

    let traces: Vec<u8> = vec![
        0x01, 0x0F, 0x00, 0x0A, 0x00, 0x14, 0x03, 0xFF, 0x40, 0x0F, 0xBB, 0x81,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 15,
            function_name: get_modbus_function_name(15).into(),
            address_unit_1: Some(10),
            quantity_unit_1: Some(20),
            count_unit_1: Some(3),
            register_units: RegisterUnits {
                mininum_value_register: Some(0x0F),
                maximum_value_register: Some(0x0FF),
                median_value_register: Some(0x040),
                total_value_register: Some(0x0F + 0x40 + 0xFF),
                zeros_count_register: Some(
                    ((0xFF as u16).count_zeros()
                        + (0x40 as u16).count_zeros()
                        + (0x0F as u16).count_zeros()) as i64
                )
            },
            crc_calculated: 0x81BB,
            ..Default::default()
        })
    );
}

#[test]
fn write_multiple_registers_01() {
    // 01 10 00 OA 00 04 08 00 CC 00 DD 00 EE 00 FF 6E 08

    let traces: Vec<u8> = vec![
        0x01, 0x10, 0x00, 0x0A, 0x00, 0x04, 0x8, 0x00, 0xCC, 0x00, 0xDD, 0x00, 0xEE, 0x00, 0xFF,
        0x6E, 0x08,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x10,
            function_name: get_modbus_function_name(0x10).into(),
            address_unit_1: Some(10),
            quantity_unit_1: Some(4),
            count_unit_1: Some(8),
            register_units: RegisterUnits {
                mininum_value_register: Some(0x00CC),
                maximum_value_register: Some(0x00FF),
                median_value_register: Some(229.5 as i64),
                total_value_register: Some(0xCC + 0xDD + 0xEE + 0xFF),
                zeros_count_register: Some(
                    ((0x00CC as u16).count_zeros()
                        + (0x00DD as u16).count_zeros()
                        + (0x00EE as u16).count_zeros()
                        + (0x00FF as u16).count_zeros()) as i64
                )
            },
            crc_calculated: 0x086E,
            ..Default::default()
        })
    );
}

#[test]
fn write_multiple_registers_02() {
    // 01 10 00 OA 00 03 06 00 BB 00 CC 00 DD 22 DD

    let traces: Vec<u8> = vec![
        0x01, 0x10, 0x00, 0x0A, 0x00, 0x3, 0x06, 0x00, 0xBB, 0x00, 0xCC, 0x00, 0xDD, 0x22, 0xDD,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x10,
            function_name: get_modbus_function_name(0x10).into(),
            address_unit_1: Some(10),
            quantity_unit_1: Some(3),
            count_unit_1: Some(6),
            register_units: RegisterUnits {
                mininum_value_register: Some(0x00BB),
                maximum_value_register: Some(0x00DD),
                median_value_register: Some(0x00CC),
                total_value_register: Some(0xBB + 0xCC + 0xDD),
                zeros_count_register: Some(
                    ((0x00BB as u16).count_zeros()
                        + (0x00CC as u16).count_zeros()
                        + (0x00DD as u16).count_zeros()) as i64
                )
            },
            crc_calculated: 0xDD22,
            ..Default::default()
        })
    );
}

#[test]
fn report_server_id() {
    // 01 11 C0 2C

    let traces: Vec<u8> = vec![0x01, 0x11, 0xC0, 0x2C];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x11,
            function_name: get_modbus_function_name(0x11).into(),
            crc_calculated: 0x2CC0,
            ..Default::default()
        })
    );
}

#[test]
fn read_file_record() {
    // 01 14 07 06 00 01 00 02 00 01 A4 E4

    let traces: Vec<u8> = vec![
        0x01, 0x14, 0x07, 0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0xA4, 0xE4,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x14,
            function_name: get_modbus_function_name(0x14).into(),
            crc_calculated: 0xE4A4,
            ..Default::default()
        })
    );
}

#[test]
fn write_file_record() {
    // 01 15 09 06 00 01 00 02 00 01 12 34 12 F5

    let traces: Vec<u8> = vec![
        0x01, 0x15, 0x09, 0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x12, 0x34, 0x12, 0xF5,
    ];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x15,
            function_name: get_modbus_function_name(0x15).into(),
            crc_calculated: 0xF512,
            ..Default::default()
        })
    );
}

#[test]
fn mask_write_record() {
    // 01 16 00 05 FF 00 00 0F 4A 16

    let traces: Vec<u8> = vec![0x01, 0x16, 0x00, 0x05, 0xFF, 0x00, 0x00, 0x0F, 0x4A, 0x16];
    let raw_trace = RawTraces { traces };

    assert_eq!(
        raw_trace.process(),
        Some(ProcessedTraces {
            slave_address: 1,
            function_code: 0x16,
            function_name: get_modbus_function_name(0x16).into(),
            crc_calculated: 0x164A,
            ..Default::default()
        })
    );
}
