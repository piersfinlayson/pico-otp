// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#![cfg(test)]

use crate::whitelabel::auto::*;
use alloc::format;

#[test]
fn test_valid_config() {
    let json = r#"{
        "device": {
            "vid": "0x1234",
            "pid": "0xabcd",
            "lang_id": "0x0409",
            "max_power": "0x32",
            "attributes": "0x80"
        }
    }"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_sample() {
    let json = include_str!("../../../json/sample-wl.json");

    let wl = WhiteLabelling::from_json(json);
    assert!(wl.is_ok());
    let wl = wl.unwrap();

    assert_eq!(wl.usb_vid().unwrap(), 0x1234);
    assert_eq!(wl.usb_pid().unwrap(), 0x4678);
    assert_eq!(wl.usb_bcd().unwrap(), 0x0100);
    assert_eq!(wl.usb_lang_id(), None);
    assert_eq!(wl.usb_manufacturer().unwrap(), "piers.rocks".into());
    assert_eq!(wl.usb_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.usb_serial_number().unwrap(), "1234abcd".into());
    assert_eq!(wl.usb_max_power(), None);
    assert_eq!(wl.usb_attributes(), None);
    assert_eq!(wl.usb_power_attributes(), None);

    assert_eq!(wl.scsi_vendor().unwrap(), "piersrks".into());
    assert_eq!(wl.scsi_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.scsi_version().unwrap(), "v123".into());

    assert_eq!(wl.volume_label().unwrap(), "PIERS.ROCKS".into());
    assert_eq!(wl.redirect_url().unwrap(), "https://piers.rocks/".into());
    assert_eq!(wl.redirect_name().unwrap(), "piers.rocks".into());
    assert_eq!(wl.uf2_model().unwrap(), "pico-otp".into());
    assert_eq!(wl.uf2_board_id().unwrap(), "pico-otp board id".into());
}

#[test]
fn test_realistic() {
    let json = include_str!("../../../json/test/realistic.json");

    let wl = WhiteLabelling::from_json(json);
    assert!(wl.is_ok());
    let wl = wl.unwrap();

    assert_eq!(wl.usb_vid().unwrap(), 0x1234);
    assert_eq!(wl.usb_pid().unwrap(), 0x4678);
    assert_eq!(wl.usb_bcd().unwrap(), 0x0100);
    assert_eq!(wl.usb_lang_id(), None);
    assert_eq!(wl.usb_manufacturer().unwrap(), "piers.rocks".into());
    assert_eq!(wl.usb_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.usb_serial_number().unwrap(), "1234abcd".into());
    assert_eq!(wl.usb_max_power(), None);
    assert_eq!(wl.usb_attributes(), None);
    assert_eq!(wl.usb_power_attributes(), None);

    assert_eq!(wl.scsi_vendor().unwrap(), "piersrks".into());
    assert_eq!(wl.scsi_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.scsi_version().unwrap(), "v123".into());

    assert_eq!(wl.volume_label().unwrap(), "PIERS.ROCKS".into());
    assert_eq!(wl.redirect_url().unwrap(), "https://piers.rocks/".into());
    assert_eq!(wl.redirect_name().unwrap(), "piers.rocks".into());
    assert_eq!(wl.uf2_model().unwrap(), "pico-otp".into());
    assert_eq!(wl.uf2_board_id().unwrap(), "pico-otp board id".into());
}

#[test]
fn test_basic() {
    let json = include_str!("../../../json/test/basic.json");

    let wl = WhiteLabelling::from_json(json);
    assert!(wl.is_ok());
    let wl = wl.unwrap();

    assert_eq!(wl.usb_vid(), None);
    assert_eq!(wl.usb_pid(), None);
    assert_eq!(wl.usb_bcd(), None);
    assert_eq!(wl.usb_lang_id(), None);
    assert_eq!(wl.usb_manufacturer().unwrap(), "piers.rocks".into());
    assert_eq!(wl.usb_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.usb_serial_number().unwrap(), "1234abcd".into());
    assert_eq!(wl.usb_max_power(), None);
    assert_eq!(wl.usb_attributes(), None);
    assert_eq!(wl.usb_power_attributes(), None);

    assert_eq!(wl.scsi_vendor().unwrap(), "piersrks".into());
    assert_eq!(wl.scsi_product().unwrap(), "pico-otp".into());
    assert_eq!(wl.scsi_version().unwrap(), "v123".into());

    assert_eq!(wl.volume_label().unwrap(), "PIERS.ROCKS".into());
    assert_eq!(wl.redirect_url().unwrap(), "https://piers.rocks/".into());
    assert_eq!(wl.redirect_name().unwrap(), "piers.rocks".into());
    assert_eq!(wl.uf2_model().unwrap(), "pico-otp".into());
    assert_eq!(wl.uf2_board_id().unwrap(), "pico-otp board id".into());
}

#[test]
fn test_utf16() {
    let json = include_str!("../../../json/test/utf16.json");

    let wl = WhiteLabelling::from_json(json);
    assert!(wl.is_ok());
    let wl = wl.unwrap();

    assert_eq!(wl.usb_manufacturer().unwrap(), "😀piers.rocks".into());
    assert_eq!(wl.usb_product().unwrap(), "pico⚡otp".into());
    assert_eq!(wl.usb_serial_number().unwrap(), "1234abcd号".into());
}

#[test]
fn test_valid_vid_format() {
    let json = r#"{"device": {"vid": "0x1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"vid": "0xABCD"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_valid_pid_format() {
    let json = r#"{"device": {"pid": "0x1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"pid": "0xABCD"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_vid_format() {
    // Missing 0x
    let json = r#"{"device": {"vid": "1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Too short
    let json = r#"{"device": {"vid": "0x123"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Too large
    let json = r#"{"device": {"vid": "0x12345"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Invalid hex
    let json = r#"{"device": {"vid": "0x12G4"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Upper case hex
    let json = r#"{"device": {"vid": "0xABCD"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_pid_format() {
    // Missing 0x
    let json = r#"{"device": {"pid": "abcd"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Too short
    let json = r#"{"device": {"pid": "0xabc"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Too large
    let json = r#"{"device": {"pid": "0xabcde"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Invalid hex
    let json = r#"{"device": {"pid": "0xabcz"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    // Upper case hex
    let json = r#"{"device": {"pid": "0xABCD"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_valid_bcd() {
    let json = r#"{"device": {"bcd": 0.0}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"bcd": 45.67}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"bcd": 99.0}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_bcd() {
    let json = r#"{"device": {"bcd": -1.0}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"bcd": 100.0}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_valid_lang_id_format() {
    let json = r#"{"device": {"lang_id": "0x0409"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"lang_id": "0x0C0a"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_lang_id_format() {
    let json = r#"{"device": {"lang_id": "409"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"lang_id": "0x040"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"lang_id": "0x04090"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"lang_id": "0x04G9"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_valid_manufacturer_string() {
    let json = r#"{"device": {"manufacturer": "ACME Corp"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"manufacturer": "ÃÇMÉ Çörp"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"manufacturer": "测试公司"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"manufacturer": "ACME 🚀 Corp"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"manufacturer": "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_manufacturer_string() {
    let long_name = "A".repeat(31);
    let json = format!(r#"{{"device": {{"manufacturer": "{}"}}}}"#, long_name);
    assert!(WhiteLabelling::from_json(&json).is_err());
}

#[test]
fn test_valid_product_string() {
    let json = r#"{"device": {"product": "Super Gadget"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"product": "Süpér Gädgét"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"product": "产品测试"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"product": "Gadget 🚀 Pro"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"product": "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_product_string() {
    let long_name = "B".repeat(31);
    let json = format!(r#"{{"device": {{"product": "{}"}}}}"#, long_name);
    assert!(WhiteLabelling::from_json(&json).is_err());
}

#[test]
fn test_valid_serial_number_string() {
    let json = r#"{"device": {"serial_number": "SN12345678"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"serial_number": "SÑ1234🚀5678"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"serial_number": "序列号测试"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"serial_number": "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_serial_number_string() {
    let long_name = "SN".repeat(16);
    let json = format!(r#"{{"device": {{"serial_number": "{}"}}}}"#, long_name);
    assert!(WhiteLabelling::from_json(&json).is_err());
}

#[test]
fn test_valid_max_power_format() {
    let json = r#"{"device": {"max_power": "0x0", "attributes": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"max_power": "0xFF", "attributes": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_max_power_format() {
    let json = r#"{"device": {"max_power": "0x123", "attributes": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"max_power": "123", "attributes": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_bad_hex_u8() {
    let json = r#"{"device": {"max_power": "0xGG", "attributes": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_valid_attributes_string() {
    let json = r#"{"device": {"attributes": "0x80", "max_power": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"attributes": "0xA0", "max_power": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"attributes": "0xC0", "max_power": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());

    let json = r#"{"device": {"attributes": "0xE0", "max_power": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_ok());
}

#[test]
fn test_invalid_attributes_string() {
    let json = r#"{"device": {"attributes": "0x90", "max_power": "0x80"}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_invalid_attributes_integer_range() {
    let json = r#"{"device": {"attributes": 127, "max_power": 128}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());

    let json = r#"{"device": {"attributes": 225, "max_power": 128}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

#[test]
fn test_invalid_attributes_integer_bits() {
    let json = r#"{"device": {"attributes": 129, "max_power": 128}}"#;
    assert!(WhiteLabelling::from_json(json).is_err());
}

fn test_ascii(a: &str, b: &str, value: &str) -> bool {
    let json = format!(r#"{{"{a}": {{"{b}": "{value}"}}}}"#);
    let result = WhiteLabelling::from_json(&json);
    result.is_ok()
}

fn test_len(a: &str, b: &str, len: usize) -> bool {
    let s = "A".repeat(len);
    let json = format!(r#"{{"{a}": {{"{b}": "{}"}}}}"#, s);
    let result = WhiteLabelling::from_json(&json);
    result.is_ok()
}

fn test_valid_scsi(field: &str, max_len: usize) {
    let max_str = "A".repeat(max_len);
    let min_str = "";
    let mid_str = "abc";
    let ok = test_ascii("scsi", field, &max_str);
    assert!(ok);

    let ok = test_ascii("scsi", field, mid_str);
    assert!(ok);

    let ok = test_ascii("scsi", field, min_str);
    assert!(ok);

    let ok = test_len("scsi", field, max_len);
    assert!(ok);
}

fn test_invalid_scsi(field: &str, max_len: usize) {
    let too_long_str = "A".repeat(max_len + 1);
    let ok = test_ascii("scsi", field, &too_long_str);
    assert!(!ok);

    let ok = test_ascii("scsi", field, "🚀");
    assert!(!ok);

    let ok = test_ascii("scsi", field, "测试");
    assert!(!ok);
}

#[test]
fn test_scsi_product() {
    test_valid_scsi("product", 16);
    test_invalid_scsi("product", 16);
}

#[test]
fn test_scsi_vendor() {
    test_valid_scsi("vendor", 8);
    test_invalid_scsi("vendor", 8);
}

#[test]
fn test_scsi_version() {
    test_valid_scsi("version", 4);
    test_invalid_scsi("version", 4);
}

fn test_valid_volume(field: &str, max_len: usize) {
    let max_str = "A".repeat(max_len);
    let min_str = "";
    let mid_str = "abc";
    let ok = test_ascii("volume", field, &max_str);
    assert!(ok);

    let ok = test_ascii("volume", field, mid_str);
    assert!(ok);

    let ok = test_ascii("volume", field, min_str);
    assert!(ok);

    let ok = test_len("volume", field, max_len);
    assert!(ok);
}

fn test_invalid_volume(field: &str, max_len: usize) {
    let too_long_str = "A".repeat(max_len + 1);
    let ok = test_ascii("volume", field, &too_long_str);
    assert!(!ok);

    let ok = test_ascii("volume", field, "🚀");
    assert!(!ok);

    let ok = test_ascii("volume", field, "测试");
    assert!(!ok);
}

#[test]
fn test_volume_board_id() {
    test_valid_volume("board_id", 127);
    test_invalid_volume("board_id", 127);
}

#[test]
fn test_volume_label() {
    test_valid_volume("label", 11);
    test_invalid_volume("label", 11);
}
#[test]
fn test_volume_model() {
    test_valid_volume("model", 127);
    test_invalid_volume("model", 127);
}
#[test]
fn test_volume_redirect_name() {
    test_valid_volume("redirect_name", 127);
    test_invalid_volume("redirect_name", 127);
}
#[test]
fn test_volume_redirect_url() {
    test_valid_volume("redirect_url", 127);
    test_invalid_volume("redirect_url", 127);
}
