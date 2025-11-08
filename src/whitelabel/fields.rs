// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Definitions of fields in the white label struct.

use alloc::format;
use alloc::string::String;

use crate::WhiteLabelStruct;

// No OTP string field can be longer than this.
pub const MAX_STRING_LENGTH: usize = 127;

/// Kind of field in the white label struct.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FieldKind {
    /// String supporting both ASCII and UTF-16, with the given maximum
    /// number of characters.
    StringUtf16AndAscii(usize),

    /// String supporting only ASCII, with the given maximum number of
    /// characters.
    StringAsciiOnly(usize),

    /// u16 field.
    U16,
}

/// Definition of a field in the white label struct.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Field {
    index: usize,
    name: &'static str,
    kind: FieldKind,
}

impl Field {
    /// Returns the name of the field.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the index of the field.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns true if the field is a u16 field.
    pub fn is_u16(&self) -> bool {
        matches!(self.kind, FieldKind::U16)
    }

    /// Returns true if the field is a string field.
    pub fn is_string(&self) -> bool {
        !self.is_u16()
    }

    /// Returns true if the field supports UTF-16 encoding (and is a string
    /// field).
    pub fn supports_utf16(&self) -> bool {
        matches!(self.kind, FieldKind::StringUtf16AndAscii(_))
    }

    /// Returns the maximum length of the field, if it is a string field.
    pub fn max_length(&self) -> Option<usize> {
        match &self.kind {
            FieldKind::StringUtf16AndAscii(len) => Some(*len),
            FieldKind::StringAsciiOnly(len) => Some(*len),
            FieldKind::U16 => None,
        }
    }

    /// Validates the field value in the given white label struct.
    pub fn validate(&self, wls: &WhiteLabelStruct) -> Result<(), String> {
        if self.is_string() {
            self.validate_string_field(wls)
        } else {
            self.validate_u16_field(wls)
        }
    }

    fn validate_u16_field(&self, wls: &WhiteLabelStruct) -> Result<(), String> {
        assert!(self.is_u16());
        match self.name() {
            "usb_vendor_id" => Ok(()),
            "usb_product_id" => Ok(()),
            "usb_language_id" => Ok(()),
            "usb_attr_power" => {
                if let Some(attr_power) = wls.attr_power() {
                    let attr = attr_power & 0xFF;
                    if (attr & 0x80) == 0 || (attr & 0x1F) != 0 {
                        return Err(format!("Invalid usb_attr_power {:#04X}", attr));
                    }

                    let power = (attr_power >> 8) & 0xFF;
                    let max_power = if (attr & 0x40) != 0 { 510 } else { 500 };
                    if power == 0 || power > max_power {
                        return Err(format!(
                            "Invalid usb_attr_power: power must be between 1 and {max_power} mA, got {power} mA",
                        ));
                    }
                }
                Ok(())
            }
            "usb_bcd_device" => {
                if let Some(bcd) = wls.bcd_device() {
                    let major = (bcd >> 8) & 0xFF;
                    let minor = (bcd >> 4) & 0x0F;
                    let patch = bcd & 0x0F;
                    if major > 99 || minor > 9 || patch > 9 {
                        return Err(format!("Invalid usb_bcd_device: {:04X} (max 99.9.9)", bcd));
                    }
                }
                Ok(())
            }
            _ => panic!("Unknown u16 field"),
        }
    }

    fn validate_string_field(&self, wls: &WhiteLabelStruct) -> Result<(), String> {
        assert!(self.is_string());

        let max_len = self.max_length().unwrap();
        let field_name = self.name();

        let field_value = match field_name {
            "usb_manufacturer" => wls.manufacturer(),
            "usb_product" => wls.product(),
            "usb_serial_number" => wls.serial_number(),
            "volume_label" => wls.volume_label(),
            "scsi_vendor" => wls.scsi_vendor(),
            "scsi_product" => wls.scsi_product(),
            "scsi_version" => wls.scsi_version(),
            "uf2_model" => wls.uf2_model(),
            "uf2_board_id" => wls.uf2_board_id(),
            "redirect_url" => wls.redirect_url(),
            "redirect_name" => wls.redirect_name(),
            _ => panic!("Unknown field"),
        };

        if let Some(value) = field_value {
            let value_len = value.chars().count();
            if value_len == 0 {
                return Err(format!("Field '{field_name}' is an empty string",));
            }
            if value_len > max_len {
                return Err(format!(
                    "Field '{field_name}' is too long: max length is {max_len}, got {value_len}",
                ));
            }

            if !self.supports_utf16() {
                if !value.is_ascii() {
                    return Err(format!(
                        "Field '{field_name}' contains non-ASCII characters",
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Number of fields in the white label struct.
pub const NUM_FIELDS: usize = 16;

pub const FIELD_USB_VENDOR_ID: Field = Field {
    index: 0,
    name: "usb_vendor_id",
    kind: FieldKind::U16,
};
pub const FIELD_USB_PRODUCT_ID: Field = Field {
    index: 1,
    name: "usb_product_id",
    kind: FieldKind::U16,
};
pub const FIELD_USB_BCD_DEVICE: Field = Field {
    index: 2,
    name: "usb_bcd_device",
    kind: FieldKind::U16,
};
pub const FIELD_USB_LANGUAGE_ID: Field = Field {
    index: 3,
    name: "usb_language_id",
    kind: FieldKind::U16,
};
pub const FIELD_USB_MANUFACTURER: Field = Field {
    index: 4,
    name: "usb_manufacturer",
    kind: FieldKind::StringUtf16AndAscii(30),
};
pub const FIELD_USB_PRODUCT: Field = Field {
    index: 5,
    name: "usb_product",
    kind: FieldKind::StringUtf16AndAscii(30),
};
pub const FIELD_USB_SERIAL_NUMBER: Field = Field {
    index: 6,
    name: "usb_serial_number",
    kind: FieldKind::StringUtf16AndAscii(30),
};
pub const FIELD_USB_ATTR_POWER: Field = Field {
    index: 7,
    name: "usb_attr_power",
    kind: FieldKind::U16,
};
pub const FIELD_VOLUME_LABEL: Field = Field {
    index: 8,
    name: "volume_label",
    kind: FieldKind::StringAsciiOnly(11),
};
pub const FIELD_SCSI_VENDOR: Field = Field {
    index: 9,
    name: "scsi_vendor",
    kind: FieldKind::StringAsciiOnly(8),
};
pub const FIELD_SCSI_PRODUCT: Field = Field {
    index: 10,
    name: "scsi_product",
    kind: FieldKind::StringAsciiOnly(16),
};
pub const FIELD_SCSI_VERSION: Field = Field {
    index: 11,
    name: "scsi_version",
    kind: FieldKind::StringAsciiOnly(4),
};
pub const FIELD_REDIRECT_URL: Field = Field {
    index: 12,
    name: "redirect_url",
    kind: FieldKind::StringAsciiOnly(127),
};
pub const FIELD_REDIRECT_NAME: Field = Field {
    index: 13,
    name: "redirect_name",
    kind: FieldKind::StringAsciiOnly(127),
};
pub const FIELD_UF2_MODEL: Field = Field {
    index: 14,
    name: "uf2_model",
    kind: FieldKind::StringAsciiOnly(127),
};
pub const FIELD_UF2_BOARD_ID: Field = Field {
    index: 15,
    name: "uf2_board_id",
    kind: FieldKind::StringAsciiOnly(127),
};

/// White label struct fields.
pub const FIELDS: [Field; NUM_FIELDS] = [
    FIELD_USB_VENDOR_ID,
    FIELD_USB_PRODUCT_ID,
    FIELD_USB_BCD_DEVICE,
    FIELD_USB_LANGUAGE_ID,
    FIELD_USB_MANUFACTURER,
    FIELD_USB_PRODUCT,
    FIELD_USB_SERIAL_NUMBER,
    FIELD_USB_ATTR_POWER,
    FIELD_VOLUME_LABEL,
    FIELD_SCSI_VENDOR,
    FIELD_SCSI_PRODUCT,
    FIELD_SCSI_VERSION,
    FIELD_REDIRECT_URL,
    FIELD_REDIRECT_NAME,
    FIELD_UF2_MODEL,
    FIELD_UF2_BOARD_ID,
];
