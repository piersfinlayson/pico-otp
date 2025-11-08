// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! pico-otp
//!
//! A library for generating and decoding OTP (One-Time Programmable) binary
//! data for the Raspberry Pi Pico 2 and other RP2350-based devices.
//!
//! This library provides functionality to create OTP binary blobs that can be
//! used to configure device-specific settings such as USB identifiers, SCSI
//! information, and volume labels. It supports whitelabel configurations
//! defined in JSON format, allowing for easy customization of device
//! parameters.
//!
//! The [`picoboot`](https://docs.rs/picoboot) crate can be used to write the
//! generated OTP data to the device, and retrieve OTP data from a device for
//! parsing by this crate, using a USB connection to the RP2350 based device in
//! BOOTSEL mode.
//!
//! It can also be used to decode OTP binary data back into native Rust objects
//! and Raspberry Pi's standard USB white labelling JSON schema.
//!
//! It is `no_std` compatible, making it suitable for WASM, embedded and other
//! minimal environments.
//!
//! It is used by [`pico⚡flash`](https://picoflash.org) to apply whitelabelling
//! configurations to Raspberry Pi Pico 2 and other RP2350 devices.
//!
//! # Features
//!
//! - Generate OTP binary data from whitelabel JSON configurations.
//! - Parses and validates OTP binary data dumps, extracting whitelabel
//!   information.
//! - Supports `picotool` whitelabel schema.
//! - `no_std` compatible (requires `alloc`).
//!
//! # Example - JSON fragment to OTP data
//!
//! ```rust
//! use pico_otp::OtpData;
//!
//! # fn main() -> Result<(), pico_otp::WhiteLabelError> {
//! // Load the JSON file
//! let json = r#"
//! {
//!    "device": {
//!       "vid": "0x1234",
//!       "pid": "0xabcd"
//!   }
//! }"#;
//!
//! // Parse it and create the OTP data object
//! let otp_data = OtpData::from_json(&json)?;
//!
//! // Generate the required OTP row data as a Vec<u16>
//! let otp_rows = otp_data.rows();
//!
//! // And the boot flags
//! let usb_boot_flags = otp_data.usb_boot_flags();
//! #   Ok(())
//! # }
//!
//! // Now, write the OTP data at a suitable row index, using ECC mode.
//! // 0x100 is a common offset choice, as it's what picotool uses.
//! //
//! // You will also need to:
//! // - Set the USB_WHITE_LABEL_ADDR row (0x05c) to point to 0x100, or
//! //   whatever row you chose.
//! // - Write usb_boot_flags to rows 0x059, 0x05a and 0x05b - USB_BOOT_FLAGS,
//! //   USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2 respectively to enable this
//! //   whitelabel data.  You must write this as non-ECC (raw) data.
//! ```
//!
//! # Example - JSON file to OTP data
//!
//! ```rust
//! use pico_otp::OtpData;
//!
//! # fn main() -> Result<(), pico_otp::WhiteLabelError> {
//! // Load the JSON file
//! let json = std::fs::read_to_string("json/sample-wl.json")
//!     .expect("Failed to read sample JSON file");
//!
//! // Parse it and create the OTP data object
//! let otp_data = OtpData::from_json(&json)?;
//!
//! // Generate the required OTP row data as a Vec<u16>
//! let otp_rows = otp_data.rows();
//!
//! // And the boot flags
//! let usb_boot_flags = otp_data.usb_boot_flags();
//! #   Ok(())
//! # }
//!
//! // Now write these to OTP memory on the RP2350.
//! ```
//!
//! # Example - Rust code to OTP data
//!
//! ```rust
//! use pico_otp::{WhiteLabelStruct, OtpData};
//!
//! // Create the WhiteLabelStruct, customizing every possible value,
//! // including Unicode strings for those that support it.
//! # fn main() -> Result<(), pico_otp::WhiteLabelError> {
//! let mut wls = WhiteLabelStruct::default();
//! wls.set_vid(0x1234);
//! wls.set_pid(0x5678);
//! wls.set_manufacturer("My Company 😀")?;
//! wls.set_product("My Product 号")?;
//! wls.set_serial_number("SN123456 🚀")?;
//! wls.set_bcd_device(0x0200);
//! wls.set_attr_power(0x80, 0xfa);
//! wls.set_scsi_vendor("MYVEND")?;
//! wls.set_scsi_product("MYPROD")?;
//! wls.set_scsi_version("1.00")?;
//! wls.set_volume_label("MYVOLUME")?;
//! wls.set_redirect_url("https://example.com")?;
//! wls.set_redirect_name("MYREDIRECT")?;
//! wls.set_uf2_model("My UF2 Model")?;
//! wls.set_uf2_board_id("My UF2 Board")?;
//!
//! // Generate the OTP data from it
//! let otp_data = wls.to_otp_data_strict()?;
//!
//! // Get the USB boot flags and ECC rows as LE bytes
//! let usb_boot_flags = otp_data.usb_boot_flags();
//! let ecc_rows = otp_data.to_le_ecc_bytes();
//! #   Ok(())
//! # }
//!
//! // Now write these to OTP memory on the RP2350.
//! ```

#![no_std]

extern crate alloc;

pub mod whitelabel;
pub use whitelabel::{Error as WhiteLabelError, OtpData, WhiteLabelStruct};

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_vid() {
        let mut wls = WhiteLabelStruct::default();
        wls.set_vid(0x1234);
        assert!(wls.is_clean());
        assert_eq!(wls.vid(), Some(0x1234));
        let od = OtpData::try_from(wls);
        assert!(od.is_ok());
    }

    #[test]
    fn test_pid() {
        let mut wls = WhiteLabelStruct::default();
        wls.set_pid(0x5678);
        assert!(wls.is_clean());
        assert_eq!(wls.pid(), Some(0x5678));
        let od = OtpData::try_from(wls);
        assert!(od.is_ok());
    }

    #[test]
    fn test_bcd() {
        let mut wls = WhiteLabelStruct::default();
        wls.set_bcd_device(0x0200);
        assert!(wls.is_clean());
        assert_eq!(wls.bcd_device(), Some(0x0200));
        let od = OtpData::try_from(wls);
        assert!(od.is_ok());
    }

    fn test_string(
        field: &str,
        good: bool,
        to_string_failure: bool,
        value: &str,
    ) -> Result<(), WhiteLabelError> {
        let mut wls = WhiteLabelStruct::default();

        // We have to handle the set_xxx() call failing if the string is > 127
        // chars - expected when to_string_failure is true
        match match field {
            "manufacturer" => wls.set_manufacturer(value.to_string()),
            "product" => wls.set_product(value.to_string()),
            "serial_number" => wls.set_serial_number(value.to_string()),
            "volume_label" => wls.set_volume_label(value.to_string()),
            "scsi_vendor" => wls.set_scsi_vendor(value.to_string()),
            "scsi_product" => wls.set_scsi_product(value.to_string()),
            "scsi_version" => wls.set_scsi_version(value.to_string()),
            "uf2_model" => wls.set_uf2_model(value.to_string()),
            "uf2_board_id" => wls.set_uf2_board_id(value.to_string()),
            "redirect_url" => wls.set_redirect_url(value.to_string()),
            "redirect_name" => wls.set_redirect_name(value.to_string()),
            _ => panic!("Unknown field"),
        } {
            Ok(_) => {
                if to_string_failure {
                    panic!("Expected failure setting field '{field}' with value '{value}'");
                }
                ()
            }
            Err(e) => {
                if to_string_failure {
                    return Ok(());
                } else {
                    panic!("Unexpected failure setting field '{field}': {e}");
                }
            }
        }
        if good {
            assert!(wls.is_clean());
        } else {
            assert!(!wls.is_clean());
        }
        assert_eq!(
            match field {
                "manufacturer" => wls.manufacturer(),
                "product" => wls.product(),
                "serial_number" => wls.serial_number(),
                "volume_label" => wls.volume_label(),
                "scsi_vendor" => wls.scsi_vendor(),
                "scsi_product" => wls.scsi_product(),
                "scsi_version" => wls.scsi_version(),
                "uf2_model" => wls.uf2_model(),
                "uf2_board_id" => wls.uf2_board_id(),
                "redirect_url" => wls.redirect_url(),
                "redirect_name" => wls.redirect_name(),
                _ => panic!("Unknown field"),
            },
            Some(&value.to_string())
        );
        let od = OtpData::try_from(wls);
        if good {
            assert!(od.is_ok());
        } else {
            assert!(od.is_err());
        }
        Ok(())
    }

    fn do_string_test(field: &str, unicode: bool, max_len: usize) {
        let res = test_string(field, unicode, false, "😀");
        assert!(res.is_ok());
        let res = test_string(field, unicode, false, "号");
        assert!(res.is_ok());
        let res = test_string(field, true, false, &"a".repeat(max_len));
        assert!(res.is_ok());

        // Empty string creates a warning
        let res = test_string(field, false, false, "");
        assert!(res.is_ok());

        // A string longer than 127 will be rejected on creation, so we have
        // to handle it differently
        let too_long = max_len + 1;
        let to_string_failure = if too_long > 127 { true } else { false };
        let res = test_string(field, false, to_string_failure, &"a".repeat(too_long));
        assert!(res.is_ok());
    }

    #[test]
    fn test_manufacturer() {
        do_string_test("manufacturer", true, 30);
    }

    #[test]
    fn test_product() {
        do_string_test("product", true, 30);
    }

    #[test]
    fn test_serial_number() {
        do_string_test("serial_number", true, 30);
    }

    #[test]
    fn test_volume_label() {
        do_string_test("volume_label", false, 11);
    }

    #[test]
    fn test_scsi_vendor() {
        do_string_test("scsi_vendor", false, 8);
    }

    #[test]
    fn test_scsi_product() {
        do_string_test("scsi_product", false, 16);
    }

    #[test]
    fn test_scsi_version() {
        do_string_test("scsi_version", false, 4);
    }

    #[test]
    fn test_uf2_model() {
        do_string_test("uf2_model", false, 127);
    }

    #[test]
    fn test_uf2_board_id() {
        do_string_test("uf2_board_id", false, 127);
    }

    #[test]
    fn test_redirect_url() {
        do_string_test("redirect_url", false, 127);
    }

    #[test]
    fn test_redirect_name() {
        do_string_test("redirect_name", false, 127);
    }

    fn test_full_cycle_string(json: &str) {
        let wls = WhiteLabelStruct::from_json(&json).unwrap();
        assert!(wls.is_clean());
        let od = OtpData::try_from(wls);
        assert!(od.is_ok());
        let otp_data = od.unwrap();
        let json_out = otp_data.to_json().unwrap();
        let wls_out = WhiteLabelStruct::from_json(&json_out.to_string()).unwrap();
        assert!(wls_out.is_clean());
        let od_out = OtpData::try_from(wls_out);
        assert!(od_out.is_ok());
        let otp_data_out = od_out.unwrap();
        assert_eq!(otp_data.usb_boot_flags(), otp_data_out.usb_boot_flags());
        assert_eq!(otp_data.rows(), otp_data_out.rows());
        assert_eq!(otp_data, otp_data_out);
    }

    fn test_full_cycle_file(filename: &str) {
        let json = std::fs::read_to_string(filename).unwrap();
        test_full_cycle_string(&json);
    }

    #[test]
    fn test_full_cycle_files() {
        let files = [
            "json/sample-wl.json",
            "json/test/attr_power_int.json",
            "json/test/basic.json",
            "json/test/complete.json",
            "json/test/realistic.json",
        ];
        for file in files {
            test_full_cycle_file(file);
        }
    }

    #[test]
    fn test_full_cycle_more() {
        let json_strs = ["{}"];
        for json in json_strs {
            test_full_cycle_string(json);
        }
    }

    #[test]
    fn just_attr() {
        let json = r#"
        {
            "device": {
                "attributes": "0x80"
            }
        }"#;
        test_full_cycle_string(json);
    }
}
