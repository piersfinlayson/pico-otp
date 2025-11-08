// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Whitelabel JSON schema types and parsing/validation.
//!
//! Provides comprehensive validation, ensuring all fields conform to expected
//! formats and constraints.
//!
//! Implementation is fully unit tested.

use alloc::format;
use alloc::string::String;
use serde::de::Error as _;

pub(crate) mod auto;
use auto::*;
mod binary;
pub use binary::OtpData;
mod fields;
mod string;
use string::OtpString;
mod top;
pub use top::{
    OTP_ROW_UNRESERVED_END, OTP_ROW_UNRESERVED_START, OTP_ROW_USB_BOOT_FLAGS,
    OTP_ROW_USB_BOOT_FLAGS_R1, OTP_ROW_USB_BOOT_FLAGS_R2, OTP_ROW_USB_WHITE_LABEL_DATA,
    WHITE_LABEL_SCHEMA_URL, WhiteLabelStruct,
};

/// Errors that can occur while handling white label data.
#[derive(Debug)]
pub enum Error {
    /// Indicates an incorrect file format or JSON not following the supported
    /// schema.
    Json(serde_json::Error),

    /// Indicates too few rows were provided to parse the white label data.
    /// The error includes the minimum required number of rows for any parsing.
    /// However, the total number of required rows may be higher, based on the
    /// length of any strings present.
    TooFewRows(usize),

    /// Indicates too many rows were provided to parse the white label data. It
    /// suggests that the provided data isn't OTP data at all.  Only pass that
    /// function a complete dump of OTP - that is 4096 rows, all read as ECC.
    TooManyRows(usize),

    /// Invalid white label address.  Returned if the WHITE_LABEL_ADDR_VALID
    /// bit in USB_BOOT_FLAGS is not set, as it is not possible to reliably
    /// detect the OTP white label data location without it.
    InvalidWhiteLabelAddress,

    /// The white label address points to an invalid or reserved OTP row.
    InvalidWhiteLabelAddressValue(u16),

    /// Indicates at least one of the three copies of the USB boot flags in the
    /// OTP data did not match the others.
    NonMatchingUsbBootFlags,

    /// Indicates there was some error or inconsistency detected during
    /// processing OTP data.  This should not happen when processing OTP data
    /// produced by `pico-otp`, but may occur when processing data from real
    /// devices, if the OTP data is corrupted or incorrectly read.  The String
    /// contains details of any inconsistencies.
    OtpDataError(String),

    /// Indicates the white label data itself is invalid or inconsistent.  The
    /// String contains details of the problems found.  This only occurs for
    /// white label data constructed from an external OTP data source, never
    /// when generated from JSON.
    InvalidWhiteLabelData(String),

    /// Indicates an internal inconsistency was detected during processing and
    /// the operation was aborted - to avoid providing an incorrect result.
    /// This indicates a bug in the library - please report it, with as much
    /// detail as possible, including the String and the input and operation
    /// that caused the error.
    InternalInconsistency(String),

    /// Indicates the string data is longer than the maximum supported.
    StringTooLong(usize),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::TooFewRows(n) => {
                write!(f, "Too few OTP rows provided: need at least {n}, got fewer")
            }
            Error::TooManyRows(n) => {
                write!(f, "Too many OTP rows provided: expected {n}, got more")
            }
            Error::InvalidWhiteLabelAddress => {
                write!(f, "WHITE_LABEL_ADDR_VALID bit not set in USB_BOOT_FLAGS")
            }
            Error::InvalidWhiteLabelAddressValue(row) => {
                write!(f, "WHITE_LABEL_ADDR points to invalid OTP row: {row}")
            }
            Error::NonMatchingUsbBootFlags => {
                write!(f, "The copies of USB_BOOT_FLAGS do not match")
            }
            Error::OtpDataError(s) => write!(f, "Invalid or inconsistent OTP data: {s}"),
            Error::InvalidWhiteLabelData(s) => write!(f, "Invalid white label data: {s}"),
            Error::InternalInconsistency(s) => write!(
                f,
                "Internal inconsistency detected and the operation was aborted.  Please report as a bug: {s}"
            ),
            Error::StringTooLong(len) => write!(
                f,
                "String is too long: maximum supported length is {}, got {len}",
                fields::MAX_STRING_LENGTH,
            ),
        }
    }
}

impl WhiteLabelling {
    /// Parse and validate a Whitelabelling configuration from a JSON string.
    ///
    /// # Arguments
    /// * `json_str` - A string slice containing the JSON representation of the
    ///   whitelabelling configuration.
    ///
    /// # Returns
    /// * `Ok(WhiteLabelling)` if parsing and validation succeed.
    /// * `Err(serde_json::Error)` if parsing or validation fail.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pico_otp::WhiteLabelStruct;
    ///
    /// // See `json/sample-wl.json` for an fuller example JSON file.
    /// let json_str = r#"{
    ///    "device": {
    ///       "vid": "0x1234",
    ///       "pid": "0xabcd"
    ///   }
    /// }"#;
    /// let wl = WhiteLabelStruct::from_json(json_str);
    /// match wl {
    ///     Ok(config) => println!("Parsed white label config: {:?}", config),
    ///     Err(e) => eprintln!("Failed to parse white label config: {}", e),
    /// }
    /// ```
    pub(crate) fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        parse_json(json_str)
    }

    /// Returns the Vendor ID (VID) as a u16, if set.
    pub(crate) fn usb_vid(&self) -> Option<u16> {
        self.device
            .as_ref()?
            .vid
            .as_ref()
            .and_then(|vid_str| u16::from_str_radix(&vid_str[2..], 16).ok())
    }

    /// Returns the Product ID (PID) as a u16, if set.
    pub(crate) fn usb_pid(&self) -> Option<u16> {
        self.device
            .as_ref()?
            .pid
            .as_ref()
            .and_then(|pid_str| u16::from_str_radix(&pid_str[2..], 16).ok())
    }

    /// Returns the BCD device revision, if set.
    pub(crate) fn usb_bcd(&self) -> Option<u16> {
        self.device.as_ref()?.bcd.map(|bcd_f| {
            // Top byte is integer part, bottom byte is fractional part
            let int_part = (bcd_f.abs() as u32) % 100;
            let frac_part = ((bcd_f.abs() - int_part as f64) * 100.0) as u32 % 100;

            // Convert to BCD
            let int_tens = (int_part / 10) as u16;
            let int_ones = (int_part % 10) as u16;
            let frac_tens = (frac_part / 10) as u16;
            let frac_ones = (frac_part % 10) as u16;

            let top_byte = (int_tens << 4) | int_ones;
            let bottom_byte = (frac_tens << 4) | frac_ones;

            (top_byte << 8) | bottom_byte
        })
    }

    /// Returns the Language ID as a u16, if set.
    pub(crate) fn usb_lang_id(&self) -> Option<u16> {
        self.device
            .as_ref()?
            .lang_id
            .as_ref()
            .and_then(|lang_id_str| u16::from_str_radix(&lang_id_str[2..], 16).ok())
    }

    /// Returns the manufacturer string, if set.
    pub(crate) fn usb_manufacturer(&self) -> Option<OtpString> {
        self.device
            .as_ref()?
            .manufacturer
            .as_deref()
            .map(OtpString::from_pre_validated_string)
    }

    /// Returns the product string, if set.
    pub(crate) fn usb_product(&self) -> Option<OtpString> {
        self.device
            .as_ref()?
            .product
            .as_deref()
            .map(OtpString::from_pre_validated_string)
    }

    /// Returns the serial number string, if set.
    pub(crate) fn usb_serial_number(&self) -> Option<OtpString> {
        self.device
            .as_ref()?
            .serial_number
            .as_deref()
            .map(OtpString::from_pre_validated_string)
    }

    /// Returns the max power as a u8, if set.
    fn usb_max_power(&self) -> Option<u8> {
        self.device
            .as_ref()?
            .max_power
            .as_ref()
            .and_then(|mp| match mp {
                WhiteLabellingDeviceMaxPower::String(s) => u8::from_str_radix(&s[2..], 16).ok(),
                WhiteLabellingDeviceMaxPower::Integer(i) => Some(*i as u8), // Already validated
            })
    }

    /// Returns the attributes as a u8, if set.
    fn usb_attributes(&self) -> Option<u8> {
        self.device
            .as_ref()?
            .attributes
            .as_ref()
            .and_then(|attr| match attr {
                WhiteLabellingDeviceAttributes::String(s) => u8::from_str_radix(&s[2..], 16).ok(),
                WhiteLabellingDeviceAttributes::Integer(i) => Some(*i as u8), // Already validated
            })
    }

    /// Returns combined USB max power and attributes byte, if both are set.
    pub(crate) fn usb_power_attributes(&self) -> Option<u16> {
        let max_power = self.usb_max_power()? as u16;
        let attributes = self.usb_attributes()? as u16;
        Some((max_power << 8) | attributes)
    }

    /// Returns the SCSI vendor string, if set.
    pub(crate) fn scsi_vendor(&self) -> Option<OtpString> {
        self.scsi.as_ref()?.vendor.as_deref().map(|s| {
            assert!(s.is_ascii(), "SCSI vendor must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the SCSI product string, if set.
    pub(crate) fn scsi_product(&self) -> Option<OtpString> {
        self.scsi.as_ref()?.product.as_deref().map(|s| {
            assert!(s.is_ascii(), "SCSI product must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the SCSI version string, if set.
    pub(crate) fn scsi_version(&self) -> Option<OtpString> {
        self.scsi.as_ref()?.version.as_deref().map(|s| {
            assert!(s.is_ascii(), "SCSI version must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the volume label string, if set.
    pub(crate) fn volume_label(&self) -> Option<OtpString> {
        self.volume.as_ref()?.label.as_deref().map(|s| {
            assert!(s.is_ascii(), "Volume label must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the volume model string, if set.
    pub(crate) fn uf2_model(&self) -> Option<OtpString> {
        self.volume.as_ref()?.model.as_deref().map(|s| {
            assert!(s.is_ascii(), "UF2 model must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the volume board ID string, if set.
    pub(crate) fn uf2_board_id(&self) -> Option<OtpString> {
        self.volume.as_ref()?.board_id.as_deref().map(|s| {
            assert!(s.is_ascii(), "UF2 board ID must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the volume redirect name string, if set.
    pub(crate) fn redirect_name(&self) -> Option<OtpString> {
        self.volume.as_ref()?.redirect_name.as_deref().map(|s| {
            assert!(s.is_ascii(), "Redirect name must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }

    /// Returns the volume redirect URL string, if set.
    pub(crate) fn redirect_url(&self) -> Option<OtpString> {
        self.volume.as_ref()?.redirect_url.as_deref().map(|s| {
            assert!(s.is_ascii(), "Redirect URL must be ASCII");
            OtpString::from_pre_validated_string(s)
        })
    }
}

fn parse_json(json_str: &str) -> Result<WhiteLabelling, serde_json::Error> {
    let mut wl: WhiteLabelling = serde_json::from_str(json_str)?;

    // Perform any corrections needed from JSON
    correct(&mut wl);

    // Validate it
    validate(&wl)?;

    Ok(wl)
}

fn correct(wl: &mut WhiteLabelling) {
    // If power provided by not attributes, set default attributes
    if let Some(device) = wl.device.as_mut()
        && device.max_power.is_some()
        && device.attributes.is_none()
    {
        device.attributes = Some(0x80.into())
    }

    // If attributes provided by not power, set default power
    if let Some(device) = wl.device.as_mut()
        && device.attributes.is_some()
        && device.max_power.is_none()
    {
        device.max_power = Some(0xfa.into()) // 500mA
    }
}

fn validate(wl: &WhiteLabelling) -> Result<(), serde_json::Error> {
    if let Some(device) = &wl.device {
        if let Some(vid) = &device.vid {
            validate_hex_u16(vid, "vid")?;
        }
        if let Some(pid) = &device.pid {
            validate_hex_u16(pid, "pid")?;
        }
        if let Some(lang_id) = &device.lang_id {
            validate_hex_u16(lang_id, "lang_id")?;
        }
        if let Some(bcd) = &device.bcd {
            validate_device_revision(*bcd, "bcd")?;
        }
        if let Some(max_power) = &device.max_power {
            match max_power {
                WhiteLabellingDeviceMaxPower::String(s) => validate_hex_u8(s, "max_power")?,
                WhiteLabellingDeviceMaxPower::Integer(_) => {} // Already validated by type
            }
            if device.attributes.is_none() {
                return Err(serde_json::Error::custom(
                    "max_power requires attributes to be set",
                ));
            }
        }
        if let Some(attributes) = &device.attributes {
            match attributes {
                WhiteLabellingDeviceAttributes::String(s) => validate_attributes_str(s)?,
                WhiteLabellingDeviceAttributes::Integer(i) => validate_attributes_int(*i)?,
            }
            if device.max_power.is_none() {
                return Err(serde_json::Error::custom(
                    "attributes requires max_power to be set",
                ));
            }
        }
    }
    if let Some(scsi) = &wl.scsi {
        if let Some(product) = &scsi.product {
            validate_ascii(product, "scsi.product")?;
        }
        if let Some(version) = &scsi.version {
            validate_ascii(version, "scsi.version")?;
        }
        if let Some(vendor) = &scsi.vendor {
            validate_ascii(vendor, "scsi.vendor")?;
        }
    }
    if let Some(volume) = &wl.volume {
        if let Some(board) = &volume.board_id {
            validate_ascii(board, "volume.board_id")?;
        }
        if let Some(label) = &volume.label {
            validate_ascii(label, "volume.label")?;
        }
        if let Some(model) = &volume.model {
            validate_ascii(model, "volume.model")?;
        }
        if let Some(redirect_name) = &volume.redirect_name {
            validate_ascii(redirect_name, "volume.redirect_name")?;
        }
        if let Some(redirect_url) = &volume.redirect_url {
            validate_ascii(redirect_url, "volume.redirect_url")?;
        }
    }
    Ok(())
}

fn validate_ascii(s: &str, field: &str) -> Result<(), serde_json::Error> {
    if !s.is_ascii() {
        return Err(serde_json::Error::custom(format!(
            "{} must be ASCII only, got: {}",
            field, s
        )));
    }
    Ok(())
}

fn validate_hex_u16(s: &str, field: &str) -> Result<(), serde_json::Error> {
    if s.len() != 6 || !s.starts_with("0x") {
        return Err(serde_json::Error::custom(format!(
            "{} must be 0x followed by 4 hex digits, got: {}",
            field, s
        )));
    }
    u16::from_str_radix(&s[2..], 16)
        .map_err(|_| serde_json::Error::custom(format!("{} contains invalid hex: {}", field, s)))?;
    Ok(())
}

fn validate_hex_u8(s: &str, field: &str) -> Result<(), serde_json::Error> {
    if s.len() < 3 || s.len() > 4 || !s.starts_with("0x") {
        return Err(serde_json::Error::custom(format!(
            "{} must be 0x followed by 1-2 hex digits, got: {}",
            field, s
        )));
    }
    u8::from_str_radix(&s[2..], 16)
        .map_err(|_| serde_json::Error::custom(format!("{} contains invalid hex: {}", field, s)))?;
    Ok(())
}

fn validate_attributes_str(s: &str) -> Result<(), serde_json::Error> {
    if s.len() != 4 || !s.starts_with("0x") || !s.ends_with('0') {
        return Err(serde_json::Error::custom(format!(
            "attributes must match 0x[8aceACE]0, got: {}",
            s
        )));
    }
    let c = s.chars().nth(2).unwrap();
    if !matches!(c, '8' | 'a' | 'c' | 'e' | 'A' | 'C' | 'E') {
        return Err(serde_json::Error::custom(format!(
            "attributes must match 0x[8aceACE]0, got: {}",
            s
        )));
    }
    Ok(())
}

fn validate_attributes_int(ii: i64) -> Result<(), serde_json::Error> {
    if ii < 128 || ii > 224 {
        return Err(serde_json::Error::custom(format!(
            "attributes integer must be 128-224, got: {}",
            ii
        )));
    }
    if ii & 0x1F != 0 {
        return Err(serde_json::Error::custom(format!(
            "attributes bits 0-4 must be 0, got: {}",
            ii
        )));
    }
    Ok(())
}

fn validate_device_revision(bcd: f64, field: &str) -> Result<(), serde_json::Error> {
    if bcd < 0.0 || bcd > 99.0 {
        return Err(serde_json::Error::custom(format!(
            "{} must be in range 0.00 to 99.0, got: {}",
            field, bcd
        )));
    }
    Ok(())
}
