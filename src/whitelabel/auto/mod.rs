// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// This file was mostly auto-generated from `/json/whitelabel-schema.json`
// using the commented out `process_wl_schema()` in `/build.rs`.
//
// Before re-generating, remove the "pattern" lines from the JSON schema, as
// these will not generate working `no_std` compatible code.
//
// It was then hand-edited to be `no_std` compatible by making the following
// changes:
//
// Replace:
// - `std::string` `with alloc::string`
// - `std::borrow::Cow` with `alloc::borrow::Cow`
// - `::std::` with `::core::`
// - `<String>` with `<::alloc::string::String>`
// - `:` String, with `: ::alloc::string::String,`
//
// Add:
// ```
// extern crate alloc;
// use alloc::string::ToString;
// #[cfg(test)]
// mod tests;
// ```

extern crate alloc;
use alloc::string::ToString;
#[cfg(test)]
mod tests;

/// Error types.
pub mod error {
    /// Error from a `TryFrom` or `FromStr` implementation.
    pub struct ConversionError(::alloc::borrow::Cow<'static, str>);
    impl ::core::error::Error for ConversionError {}
    impl ::core::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> Result<(), ::core::fmt::Error> {
            ::core::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::core::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> Result<(), ::core::fmt::Error> {
            ::core::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<::alloc::string::String> for ConversionError {
        fn from(value: ::alloc::string::String) -> Self {
            Self(value.into())
        }
    }
}

///White Labelling Configuration, see section 5.7 in the RP2350 datasheet for more details
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "White Labelling",
///  "description": "White Labelling Configuration, see section 5.7 in the RP2350 datasheet for more details",
///  "type": "object",
///  "properties": {
///    "$schema": {},
///    "device": {
///      "description": "Device Properties",
///      "type": "object",
///      "properties": {
///        "attributes": {
///          "description": "Device attributes: bit 7 must be 1, bit 6 is self-powered, bit 5 is remote wakeup, bits 0-4 must be 0",
///          "type": [
///            "integer",
///            "string"
///          ],
///          "maximum": 224.0,
///          "minimum": 128.0
///        },
///        "bcd": {
///          "description": "Device Revision",
///          "type": "number",
///          "maximum": 99.0,
///          "minimum": 0.0
///        },
///        "lang_id": {
///          "description": "Language ID",
///          "type": "string"
///        },
///        "manufacturer": {
///          "description": "Manufacturer Name (can contain unicode)",
///          "type": "string",
///          "maxLength": 30
///        },
///        "max_power": {
///          "description": "Max power consumption, in 2mA units",
///          "type": [
///            "integer",
///            "string"
///          ],
///          "maximum": 255.0
///        },
///        "pid": {
///          "description": "Product ID",
///          "type": "string"
///        },
///        "product": {
///          "description": "Product Name (can contain unicode)",
///          "type": "string",
///          "maxLength": 30
///        },
///        "serial_number": {
///          "description": "Serial Number (can contain unicode)",
///          "type": "string",
///          "maxLength": 30
///        },
///        "vid": {
///          "description": "Vendor ID",
///          "type": "string"
///        }
///      },
///      "additionalProperties": false,
///      "dependentRequired": {
///        "attributes": [
///          "max_power"
///        ],
///        "max_power": [
///          "attributes"
///        ]
///      }
///    },
///    "scsi": {
///      "description": "SCSI Inquiry Values",
///      "type": "object",
///      "properties": {
///        "product": {
///          "description": "SCSI Product",
///          "type": "string",
///          "maxLength": 16
///        },
///        "vendor": {
///          "description": "SCSI Vendor",
///          "type": "string",
///          "maxLength": 8
///        },
///        "version": {
///          "description": "SCSI Version",
///          "type": "string",
///          "maxLength": 4
///        }
///      },
///      "additionalProperties": false
///    },
///    "volume": {
///      "description": "MSD Volume Configuration",
///      "type": "object",
///      "properties": {
///        "board_id": {
///          "description": "INFO_UF2.TXT Board ID",
///          "type": "string",
///          "maxLength": 127
///        },
///        "label": {
///          "description": "Volume Label",
///          "type": "string",
///          "maxLength": 11
///        },
///        "model": {
///          "description": "INFO_UF2.TXT Model Name",
///          "type": "string",
///          "maxLength": 127
///        },
///        "redirect_name": {
///          "description": "INDEX.HTM Redirect Name",
///          "type": "string",
///          "maxLength": 127
///        },
///        "redirect_url": {
///          "description": "INDEX.HTM Redirect URL",
///          "type": "string",
///          "maxLength": 127
///        }
///      },
///      "additionalProperties": false
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WhiteLabelling {
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub device: ::core::option::Option<WhiteLabellingDevice>,
    #[serde(
        rename = "$schema",
        default,
        skip_serializing_if = "::core::option::Option::is_none"
    )]
    pub schema: ::core::option::Option<::serde_json::Value>,
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub scsi: ::core::option::Option<WhiteLabellingScsi>,
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub volume: ::core::option::Option<WhiteLabellingVolume>,
}
impl ::core::convert::From<&WhiteLabelling> for WhiteLabelling {
    fn from(value: &WhiteLabelling) -> Self {
        value.clone()
    }
}
impl ::core::default::Default for WhiteLabelling {
    fn default() -> Self {
        Self {
            device: Default::default(),
            schema: Default::default(),
            scsi: Default::default(),
            volume: Default::default(),
        }
    }
}
///Device Properties
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Device Properties",
///  "type": "object",
///  "properties": {
///    "attributes": {
///      "description": "Device attributes: bit 7 must be 1, bit 6 is self-powered, bit 5 is remote wakeup, bits 0-4 must be 0",
///      "type": [
///        "integer",
///        "string"
///      ],
///      "maximum": 224.0,
///      "minimum": 128.0
///    },
///    "bcd": {
///      "description": "Device Revision",
///      "type": "number",
///      "maximum": 99.0,
///      "minimum": 0.0
///    },
///    "lang_id": {
///      "description": "Language ID",
///      "type": "string"
///    },
///    "manufacturer": {
///      "description": "Manufacturer Name (can contain unicode)",
///      "type": "string",
///      "maxLength": 30
///    },
///    "max_power": {
///      "description": "Max power consumption, in 2mA units",
///      "type": [
///        "integer",
///        "string"
///      ],
///      "maximum": 255.0
///    },
///    "pid": {
///      "description": "Product ID",
///      "type": "string"
///    },
///    "product": {
///      "description": "Product Name (can contain unicode)",
///      "type": "string",
///      "maxLength": 30
///    },
///    "serial_number": {
///      "description": "Serial Number (can contain unicode)",
///      "type": "string",
///      "maxLength": 30
///    },
///    "vid": {
///      "description": "Vendor ID",
///      "type": "string"
///    }
///  },
///  "additionalProperties": false,
///  "dependentRequired": {
///    "attributes": [
///      "max_power"
///    ],
///    "max_power": [
///      "attributes"
///    ]
///  }
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WhiteLabellingDevice {
    ///Device attributes: bit 7 must be 1, bit 6 is self-powered, bit 5 is remote wakeup, bits 0-4 must be 0
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub attributes: ::core::option::Option<WhiteLabellingDeviceAttributes>,
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub bcd: ::core::option::Option<f64>,
    ///Language ID
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub lang_id: ::core::option::Option<::alloc::string::String>,
    ///Manufacturer Name (can contain unicode)
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub manufacturer: ::core::option::Option<WhiteLabellingDeviceManufacturer>,
    ///Max power consumption, in 2mA units
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub max_power: ::core::option::Option<WhiteLabellingDeviceMaxPower>,
    ///Product ID
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub pid: ::core::option::Option<::alloc::string::String>,
    ///Product Name (can contain unicode)
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub product: ::core::option::Option<WhiteLabellingDeviceProduct>,
    ///Serial Number (can contain unicode)
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub serial_number: ::core::option::Option<WhiteLabellingDeviceSerialNumber>,
    ///Vendor ID
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub vid: ::core::option::Option<::alloc::string::String>,
}
impl ::core::convert::From<&WhiteLabellingDevice> for WhiteLabellingDevice {
    fn from(value: &WhiteLabellingDevice) -> Self {
        value.clone()
    }
}
impl ::core::default::Default for WhiteLabellingDevice {
    fn default() -> Self {
        Self {
            attributes: Default::default(),
            bcd: Default::default(),
            lang_id: Default::default(),
            manufacturer: Default::default(),
            max_power: Default::default(),
            pid: Default::default(),
            product: Default::default(),
            serial_number: Default::default(),
            vid: Default::default(),
        }
    }
}
///Device attributes: bit 7 must be 1, bit 6 is self-powered, bit 5 is remote wakeup, bits 0-4 must be 0
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Device attributes: bit 7 must be 1, bit 6 is self-powered, bit 5 is remote wakeup, bits 0-4 must be 0",
///  "type": [
///    "integer",
///    "string"
///  ],
///  "maximum": 224.0,
///  "minimum": 128.0
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum WhiteLabellingDeviceAttributes {
    String(::alloc::string::String),
    Integer(i64),
}
impl ::core::convert::From<&Self> for WhiteLabellingDeviceAttributes {
    fn from(value: &WhiteLabellingDeviceAttributes) -> Self {
        value.clone()
    }
}
impl ::core::fmt::Display for WhiteLabellingDeviceAttributes {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::String(x) => x.fmt(f),
            Self::Integer(x) => x.fmt(f),
        }
    }
}
impl ::core::convert::From<i64> for WhiteLabellingDeviceAttributes {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}
///Manufacturer Name (can contain unicode)
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Manufacturer Name (can contain unicode)",
///  "type": "string",
///  "maxLength": 30
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingDeviceManufacturer(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingDeviceManufacturer {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingDeviceManufacturer> for ::alloc::string::String {
    fn from(value: WhiteLabellingDeviceManufacturer) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingDeviceManufacturer> for WhiteLabellingDeviceManufacturer {
    fn from(value: &WhiteLabellingDeviceManufacturer) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingDeviceManufacturer {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 30usize {
            return Err("longer than 30 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingDeviceManufacturer {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingDeviceManufacturer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingDeviceManufacturer {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingDeviceManufacturer {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///Max power consumption, in 2mA units
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Max power consumption, in 2mA units",
///  "type": [
///    "integer",
///    "string"
///  ],
///  "maximum": 255.0
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum WhiteLabellingDeviceMaxPower {
    String(::alloc::string::String),
    Integer(u8),
}
impl ::core::convert::From<&Self> for WhiteLabellingDeviceMaxPower {
    fn from(value: &WhiteLabellingDeviceMaxPower) -> Self {
        value.clone()
    }
}
impl ::core::fmt::Display for WhiteLabellingDeviceMaxPower {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::String(x) => x.fmt(f),
            Self::Integer(x) => x.fmt(f),
        }
    }
}
impl ::core::convert::From<u8> for WhiteLabellingDeviceMaxPower {
    fn from(value: u8) -> Self {
        Self::Integer(value)
    }
}
///Product Name (can contain unicode)
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Product Name (can contain unicode)",
///  "type": "string",
///  "maxLength": 30
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingDeviceProduct(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingDeviceProduct {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingDeviceProduct> for ::alloc::string::String {
    fn from(value: WhiteLabellingDeviceProduct) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingDeviceProduct> for WhiteLabellingDeviceProduct {
    fn from(value: &WhiteLabellingDeviceProduct) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingDeviceProduct {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 30usize {
            return Err("longer than 30 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingDeviceProduct {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingDeviceProduct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingDeviceProduct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingDeviceProduct {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///Serial Number (can contain unicode)
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Serial Number (can contain unicode)",
///  "type": "string",
///  "maxLength": 30
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingDeviceSerialNumber(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingDeviceSerialNumber {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingDeviceSerialNumber> for ::alloc::string::String {
    fn from(value: WhiteLabellingDeviceSerialNumber) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingDeviceSerialNumber> for WhiteLabellingDeviceSerialNumber {
    fn from(value: &WhiteLabellingDeviceSerialNumber) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingDeviceSerialNumber {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 30usize {
            return Err("longer than 30 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingDeviceSerialNumber {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingDeviceSerialNumber {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingDeviceSerialNumber {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingDeviceSerialNumber {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///SCSI Inquiry Values
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "SCSI Inquiry Values",
///  "type": "object",
///  "properties": {
///    "product": {
///      "description": "SCSI Product",
///      "type": "string",
///      "maxLength": 16
///    },
///    "vendor": {
///      "description": "SCSI Vendor",
///      "type": "string",
///      "maxLength": 8
///    },
///    "version": {
///      "description": "SCSI Version",
///      "type": "string",
///      "maxLength": 4
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WhiteLabellingScsi {
    ///SCSI Product
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub product: ::core::option::Option<WhiteLabellingScsiProduct>,
    ///SCSI Vendor
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub vendor: ::core::option::Option<WhiteLabellingScsiVendor>,
    ///SCSI Version
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub version: ::core::option::Option<WhiteLabellingScsiVersion>,
}
impl ::core::convert::From<&WhiteLabellingScsi> for WhiteLabellingScsi {
    fn from(value: &WhiteLabellingScsi) -> Self {
        value.clone()
    }
}
impl ::core::default::Default for WhiteLabellingScsi {
    fn default() -> Self {
        Self {
            product: Default::default(),
            vendor: Default::default(),
            version: Default::default(),
        }
    }
}
///SCSI Product
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "SCSI Product",
///  "type": "string",
///  "maxLength": 16
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingScsiProduct(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingScsiProduct {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingScsiProduct> for ::alloc::string::String {
    fn from(value: WhiteLabellingScsiProduct) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingScsiProduct> for WhiteLabellingScsiProduct {
    fn from(value: &WhiteLabellingScsiProduct) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingScsiProduct {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 16usize {
            return Err("longer than 16 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingScsiProduct {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingScsiProduct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingScsiProduct {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingScsiProduct {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///SCSI Vendor
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "SCSI Vendor",
///  "type": "string",
///  "maxLength": 8
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingScsiVendor(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingScsiVendor {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingScsiVendor> for ::alloc::string::String {
    fn from(value: WhiteLabellingScsiVendor) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingScsiVendor> for WhiteLabellingScsiVendor {
    fn from(value: &WhiteLabellingScsiVendor) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingScsiVendor {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 8usize {
            return Err("longer than 8 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingScsiVendor {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingScsiVendor {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingScsiVendor {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingScsiVendor {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///SCSI Version
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "SCSI Version",
///  "type": "string",
///  "maxLength": 4
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingScsiVersion(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingScsiVersion {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingScsiVersion> for ::alloc::string::String {
    fn from(value: WhiteLabellingScsiVersion) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingScsiVersion> for WhiteLabellingScsiVersion {
    fn from(value: &WhiteLabellingScsiVersion) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingScsiVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 4usize {
            return Err("longer than 4 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingScsiVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingScsiVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingScsiVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingScsiVersion {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///MSD Volume Configuration
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "MSD Volume Configuration",
///  "type": "object",
///  "properties": {
///    "board_id": {
///      "description": "INFO_UF2.TXT Board ID",
///      "type": "string",
///      "maxLength": 127
///    },
///    "label": {
///      "description": "Volume Label",
///      "type": "string",
///      "maxLength": 11
///    },
///    "model": {
///      "description": "INFO_UF2.TXT Model Name",
///      "type": "string",
///      "maxLength": 127
///    },
///    "redirect_name": {
///      "description": "INDEX.HTM Redirect Name",
///      "type": "string",
///      "maxLength": 127
///    },
///    "redirect_url": {
///      "description": "INDEX.HTM Redirect URL",
///      "type": "string",
///      "maxLength": 127
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WhiteLabellingVolume {
    ///INFO_UF2.TXT Board ID
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub board_id: ::core::option::Option<WhiteLabellingVolumeBoardId>,
    ///Volume Label
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub label: ::core::option::Option<WhiteLabellingVolumeLabel>,
    ///INFO_UF2.TXT Model Name
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub model: ::core::option::Option<WhiteLabellingVolumeModel>,
    ///INDEX.HTM Redirect Name
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub redirect_name: ::core::option::Option<WhiteLabellingVolumeRedirectName>,
    ///INDEX.HTM Redirect URL
    #[serde(default, skip_serializing_if = "::core::option::Option::is_none")]
    pub redirect_url: ::core::option::Option<WhiteLabellingVolumeRedirectUrl>,
}
impl ::core::convert::From<&WhiteLabellingVolume> for WhiteLabellingVolume {
    fn from(value: &WhiteLabellingVolume) -> Self {
        value.clone()
    }
}
impl ::core::default::Default for WhiteLabellingVolume {
    fn default() -> Self {
        Self {
            board_id: Default::default(),
            label: Default::default(),
            model: Default::default(),
            redirect_name: Default::default(),
            redirect_url: Default::default(),
        }
    }
}
///INFO_UF2.TXT Board ID
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "INFO_UF2.TXT Board ID",
///  "type": "string",
///  "maxLength": 127
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingVolumeBoardId(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingVolumeBoardId {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingVolumeBoardId> for ::alloc::string::String {
    fn from(value: WhiteLabellingVolumeBoardId) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingVolumeBoardId> for WhiteLabellingVolumeBoardId {
    fn from(value: &WhiteLabellingVolumeBoardId) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingVolumeBoardId {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 127usize {
            return Err("longer than 127 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingVolumeBoardId {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingVolumeBoardId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingVolumeBoardId {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingVolumeBoardId {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///Volume Label
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Volume Label",
///  "type": "string",
///  "maxLength": 11
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingVolumeLabel(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingVolumeLabel {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingVolumeLabel> for ::alloc::string::String {
    fn from(value: WhiteLabellingVolumeLabel) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingVolumeLabel> for WhiteLabellingVolumeLabel {
    fn from(value: &WhiteLabellingVolumeLabel) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingVolumeLabel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 11usize {
            return Err("longer than 11 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingVolumeLabel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingVolumeLabel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingVolumeLabel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingVolumeLabel {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///INFO_UF2.TXT Model Name
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "INFO_UF2.TXT Model Name",
///  "type": "string",
///  "maxLength": 127
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingVolumeModel(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingVolumeModel {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingVolumeModel> for ::alloc::string::String {
    fn from(value: WhiteLabellingVolumeModel) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingVolumeModel> for WhiteLabellingVolumeModel {
    fn from(value: &WhiteLabellingVolumeModel) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingVolumeModel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 127usize {
            return Err("longer than 127 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingVolumeModel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingVolumeModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingVolumeModel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingVolumeModel {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///INDEX.HTM Redirect Name
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "INDEX.HTM Redirect Name",
///  "type": "string",
///  "maxLength": 127
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingVolumeRedirectName(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingVolumeRedirectName {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingVolumeRedirectName> for ::alloc::string::String {
    fn from(value: WhiteLabellingVolumeRedirectName) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingVolumeRedirectName> for WhiteLabellingVolumeRedirectName {
    fn from(value: &WhiteLabellingVolumeRedirectName) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingVolumeRedirectName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 127usize {
            return Err("longer than 127 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingVolumeRedirectName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingVolumeRedirectName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingVolumeRedirectName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingVolumeRedirectName {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
///INDEX.HTM Redirect URL
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "INDEX.HTM Redirect URL",
///  "type": "string",
///  "maxLength": 127
///}
/// ```
/// </details>
#[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct WhiteLabellingVolumeRedirectUrl(::alloc::string::String);
impl ::core::ops::Deref for WhiteLabellingVolumeRedirectUrl {
    type Target = ::alloc::string::String;
    fn deref(&self) -> &::alloc::string::String {
        &self.0
    }
}
impl ::core::convert::From<WhiteLabellingVolumeRedirectUrl> for ::alloc::string::String {
    fn from(value: WhiteLabellingVolumeRedirectUrl) -> Self {
        value.0
    }
}
impl ::core::convert::From<&WhiteLabellingVolumeRedirectUrl> for WhiteLabellingVolumeRedirectUrl {
    fn from(value: &WhiteLabellingVolumeRedirectUrl) -> Self {
        value.clone()
    }
}
impl ::core::str::FromStr for WhiteLabellingVolumeRedirectUrl {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 127usize {
            return Err("longer than 127 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::core::convert::TryFrom<&str> for WhiteLabellingVolumeRedirectUrl {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<&::alloc::string::String> for WhiteLabellingVolumeRedirectUrl {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::core::convert::TryFrom<::alloc::string::String> for WhiteLabellingVolumeRedirectUrl {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::alloc::string::String,
    ) -> ::core::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for WhiteLabellingVolumeRedirectUrl {
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::alloc::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
