//! Preserved status and opcode checks from catalog sx1262 §13 “Commands Interface”.
use sx1262_phy::*;

#[test]
fn decode_radio_status() {
    // Table 13-76: StbyRc (0x2 = 0b010) + DataAvailable (0x2 = 0b010):
    // bits: reserved(0) | mode(010) | cmd(010) | reserved(0) = 0b0010_0100 = 0x24
    let stby_data = RadioStatus::from_byte(0x24);
    assert_eq!(stby_data.chip_mode, ChipMode::StbyRc);
    assert_eq!(stby_data.command_status, CommandStatus::DataAvailable);
    assert!(stby_data.is_standby());
    assert!(stby_data.is_ok());

    // StbyXosc (0x3 = 0b011) + Reserved (0) = 0b0011_0000 = 0x30
    let stby_xosc = RadioStatus::from_byte(0x30);
    assert_eq!(stby_xosc.chip_mode, ChipMode::StbyXosc);
    assert!(stby_xosc.is_standby());
    assert!(stby_xosc.is_ok());

    // Error status: ExecutionFailure (0x5 = 0b101) in bits 3:1 = 0x0A
    let err_status = RadioStatus::from_byte(0x2A);
    assert_eq!(err_status.command_status, CommandStatus::ExecutionFailure);
    assert!(!err_status.is_ok());
}

#[test]
fn opcodes_match_sx1262_specification() {
    assert_eq!(CMD_GET_STATUS, 0xC0);
    assert_eq!(CMD_GET_PACKET_TYPE, 0x11);
    assert_eq!(CMD_SET_STANDBY, 0x80);
    assert_eq!(CMD_GET_DEVICE_ERRORS, 0x17);
    assert_eq!(CMD_SET_RF_FREQUENCY, 0x86);
    assert_eq!(CMD_SET_PACKET_PARAMS, 0x8C);
    assert_eq!(CMD_SET_MODULATION_PARAMS, 0x8B);
    assert_eq!(CMD_SET_DIO_IRQ_PARAMS, 0x08);
    assert_eq!(CMD_GET_IRQ_STATUS, 0x12);
    assert_eq!(CMD_CLEAR_IRQ_STATUS, 0x02);
    assert_eq!(CMD_SET_CAD, 0xC5);
    assert_eq!(CMD_SET_CAD_PARAMS, 0x88);
}

#[test]
fn airtime_covers_crc_sf5_sf6_and_large_preambles_without_overflow() {
    let modulation =
        BaseBandModulationParams::new(SpreadingFactor::_7, Bandwidth::_125KHz, CodingRate::_4_5);
    let mut packet = LoRaPacketParams {
        preamble_symbols: 8,
        implicit_header: false,
        payload_len: 13,
        crc: true,
        invert_iq: false,
    };
    assert_eq!(lora_airtime_us(&modulation, &packet), 46_336);
    packet.crc = false;
    assert_eq!(lora_airtime_us(&modulation, &packet), 41_216);
    packet.implicit_header = true;
    assert_eq!(lora_airtime_us(&modulation, &packet), 36_096);
    for sf in [SpreadingFactor::_5, SpreadingFactor::_6] {
        let m = BaseBandModulationParams::new(sf, Bandwidth::_500KHz, CodingRate::_4_5);
        let p = LoRaPacketParams {
            preamble_symbols: 1,
            implicit_header: false,
            payload_len: 0,
            crc: false,
            invert_iq: false,
        };
        let expected = if sf == SpreadingFactor::_5 {
            1680
        } else {
            3360
        };
        assert_eq!(lora_airtime_us(&m, &p), expected);
        let with_minimum = LoRaPacketParams {
            preamble_symbols: 12,
            ..p
        };
        assert_eq!(lora_airtime_us(&m, &p), lora_airtime_us(&m, &with_minimum));
    }
    let slow =
        BaseBandModulationParams::new(SpreadingFactor::_12, Bandwidth::_7KHz, CodingRate::_4_8);
    packet.preamble_symbols = u16::MAX;
    packet.payload_len = u8::MAX;
    let airtime = lora_airtime_us(&slow, &packet);
    assert!(airtime > u64::from(u32::MAX));
    assert!(airtime < 40_000_000_000);
}
