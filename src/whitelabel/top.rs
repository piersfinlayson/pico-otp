// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Contains the objects and functions related to the top level OTP USB White
//! Label object type.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;

use crate::whitelabel::auto::{WhiteLabellingDeviceManufacturer, WhiteLabellingDeviceSerialNumber, WhiteLabellingDeviceProduct, WhiteLabellingDeviceAttributes, WhiteLabellingDeviceMaxPower};
use crate::whitelabel::auto::{WhiteLabellingScsiProduct, WhiteLabellingScsiVendor, WhiteLabellingScsiVersion};
use crate::whitelabel::auto::{WhiteLabellingVolumeLabel, WhiteLabellingVolumeModel, WhiteLabellingVolumeBoardId, WhiteLabellingVolumeRedirectName, WhiteLabellingVolumeRedirectUrl};
use crate::whitelabel::{Error, OtpString, WhiteLabelling, WhiteLabellingDevice, WhiteLabellingScsi, WhiteLabellingVolume};

/// Number of rows in the white label struct that are u16 values.
pub const NUM_U16_ROWS: usize = 5;
/// Indices of the u16 rows in the white label struct.
pub const U16_ROWS: [usize; NUM_U16_ROWS] = [0, 1, 2, 3, 7];
/// Number of rows in the white label struct that are STRDEF pointers.
pub const NUM_STRDEF_ROWS: usize = 11;
/// Indices of the STRDEF rows in the white label struct.
pub const STRDEF_ROWS: [usize; NUM_STRDEF_ROWS] = [4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15];
/// Total number of rows in the white label struct.
pub const NUM_INDEX_ROWS: usize = NUM_U16_ROWS + NUM_STRDEF_ROWS;
/// White label address value valid bit index within the USB_BOOT_FLAGS
pub const WHITE_LABEL_ADDR_VALID_BIT_NUM: usize = 22;
/// DP/DM Swap bit index within the USB_BOOT_FLAGS
pub const DP_DM_SWAP_BIT_NUM: usize = 23;
/// Total number of rows the RP2350's OTP memory
pub const TOTAL_OTP_ROWS: usize = 4096;

/// OTP row index for USB_BOOT_FLAGS
pub const OTP_ROW_USB_BOOT_FLAGS: u16 = 0x059;
/// OTP row index for USB_BOOT_FLAGS_R1
pub const OTP_ROW_USB_BOOT_FLAGS_R1: u16 = 0x05a;
/// OTP row index for USB_BOOT_FLAGS_R2
pub const OTP_ROW_USB_BOOT_FLAGS_R2: u16 = 0x05b;
/// OTP row index for USB_WHITE_LABEL_DATA
pub const OTP_ROW_USB_WHITE_LABEL_DATA: u16 = 0x05c;

/// URL of the official Raspberry Pi picotool white label JSON schema,
/// supported by this crate
pub const WHITE_LABEL_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/raspberrypi/picotool/develop/json/schemas/whitelabel-schema.json";

/// Names of the fields in the white label struct, indexed by the relevant
/// bit position in [`OTP_ROW_USB_WHITE_LABEL_DATA`].
pub const FIELD_NAMES: [&str; NUM_INDEX_ROWS] = [
    "usb_vendor_id",
    "usb_product_id",
    "usb_bcd_device",
    "usb_language_id",
    "usb_manufacturer",
    "usb_product",
    "usb_serial_number",
    "usb_attr_power",
    "volume_label",
    "scsi_vendor",
    "scsi_product",
    "scsi_version",
    "redirect_url",
    "redirect_name",
    "uf2_model",
    "uf2_board_id",
];

/// Maximum string lengths for each field in the white label struct, indexed
/// by the relevant bit position in USB_WHITE_LABEL_ADDR.  A value of 0
/// indicates a u16 field.
pub const MAX_STRING_LEN: [usize; NUM_INDEX_ROWS] = [
    0,
    0,
    0,
    0,
    30,
    30,
    30,
    0,
    11,
    8,
    16,
    4,
    127,
    127,
    127,
    127,
];

/// Represents the USB white label structure stored in OTP.
///
/// Use [`WhiteLabelStruct::from_json`] to create an instance from a JSON
/// whitelabel representation, or use [`WhiteLabelStruct::from_otp`] to parse
/// an instance from OTP rows.
///
/// To create an empty instance, use [`WhiteLabelStruct::default`] and set
/// fields as required.
///
/// To create the OTP data rows required to store this white label structure,
/// use [`WhiteLabelStruct::to_otp_rows`] and for the USB_BOOT_FLAGS value,
/// [`WhiteLabelStruct::usb_boot_flags`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WhiteLabelStruct {
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub bcd_device: Option<u16>,
    pub language_id: Option<u16>,
    pub manufacturer: Option<OtpString>,
    pub product: Option<OtpString>,
    pub serial_number: Option<OtpString>,
    pub attr_power: Option<u16>,
    pub volume_label: Option<OtpString>,
    pub scsi_vendor: Option<OtpString>,
    pub scsi_product: Option<OtpString>,
    pub scsi_version: Option<OtpString>,
    pub redirect_url: Option<OtpString>,
    pub redirect_name: Option<OtpString>,
    pub uf2_model: Option<OtpString>,
    pub uf2_board_id: Option<OtpString>,
}

// Has to handle options
impl From<WhiteLabelStruct> for WhiteLabellingDevice {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut device = WhiteLabellingDevice::default();

        let vid = wls.vendor_id.as_ref().map(|v| format!("{:#06x}", v));
        let pid = wls.product_id.as_ref().map(|v| format!("{:#06x}", v));
        let bcd = wls.bcd_device.as_ref().map(|v| {
            let integer = (v >> 8) as u8;
            let tenths = (v >> 4) & 0x0F;
            let hundredths = v & 0x0F;

            integer as f64 + (tenths as f64 / 10.0) + (hundredths as f64 / 100.0)
        });
        let lang_id = wls.language_id.as_ref().map(|v| format!("{:#06x}", v));
        let manufacturer = wls.manufacturer.as_ref().map(|s| WhiteLabellingDeviceManufacturer::from_str(&s.to_string()).expect("Invalid manufacturer string"));
        let product = wls.product.as_ref().map(|s| WhiteLabellingDeviceProduct::from_str(&s.to_string()).expect("Invalid product string"));
        let serial_number = wls.serial_number.as_ref().map(|s| WhiteLabellingDeviceSerialNumber::from_str(&s.to_string()).expect("Invalid serial number string"));
        let attributes = wls.attr_power.as_ref().map(|v| WhiteLabellingDeviceAttributes::String(format!("{:#04x}", ((*v & 0xFF) as u8))));
        let max_power = wls.attr_power.as_ref().map(|v| WhiteLabellingDeviceMaxPower::String(format!("{:#04x}", (((*v & 0xFF00) >> 8) as u8))));

        device.vid = vid;
        device.pid = pid;
        device.bcd = bcd;
        device.lang_id = lang_id;
        device.manufacturer = manufacturer;
        device.product = product;
        device.serial_number = serial_number;
        device.attributes = attributes;
        device.max_power = max_power;

        device
    }
}

// Into WhiteLabellingScsi
impl From<WhiteLabelStruct> for WhiteLabellingScsi {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut scsi = WhiteLabellingScsi::default();

        let vendor = wls.scsi_vendor.as_ref().map(|s| WhiteLabellingScsiVendor::from_str(&s.to_string()).expect("Invalid SCSI vendor string"));
        let product = wls.scsi_product.as_ref().map(|s| WhiteLabellingScsiProduct::from_str(&s.to_string()).expect("Invalid SCSI product string"));
        let version = wls.scsi_version.as_ref().map(|s| WhiteLabellingScsiVersion::from_str(&s.to_string()).expect("Invalid SCSI version string"));

        scsi.vendor = vendor;
        scsi.product = product;
        scsi.version = version;

        scsi
    }
}

// Into WhiteLabellingVolume
impl From<WhiteLabelStruct> for WhiteLabellingVolume {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut volume = WhiteLabellingVolume::default();

        let label = wls.volume_label.as_ref().map(|s| WhiteLabellingVolumeLabel::from_str(&s.to_string()).expect("Invalid volume label string"));
        let model = wls.uf2_model.as_ref().map(|s| WhiteLabellingVolumeModel::from_str(&s.to_string()).expect("Invalid UF2 model string"));
        let board_id = wls.uf2_board_id.as_ref().map(|s| WhiteLabellingVolumeBoardId::from_str(&s.to_string()).expect("Invalid UF2 board ID string"));
        let redirect_name = wls.redirect_name.as_ref().map(|s| WhiteLabellingVolumeRedirectName::from_str(&s.to_string()).expect("Invalid redirect name string"));
        let redirect_url = wls.redirect_url.as_ref().map(|s| WhiteLabellingVolumeRedirectUrl::from_str(&s.to_string()).expect("Invalid redirect URL string"));

        volume.label = label;
        volume.model = model;
        volume.board_id = board_id;
        volume.redirect_name = redirect_name;
        volume.redirect_url = redirect_url;

        volume
    }
}

// Into WhiteLabelling
impl From<WhiteLabelStruct> for WhiteLabelling {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut wl = WhiteLabelling::default();

        wl.schema = Some(serde_json::Value::String(WHITE_LABEL_SCHEMA_URL.to_string()));

        // Only create device if any device fields are present
        if wls.vendor_id.is_some() || wls.product_id.is_some() || wls.bcd_device.is_some()
            || wls.language_id.is_some() || wls.manufacturer.is_some() 
            || wls.product.is_some() || wls.serial_number.is_some() || wls.attr_power.is_some() {
            wl.device = Some(WhiteLabellingDevice::from(wls.clone()));
        }

        // Only create scsi if any scsi fields are present
        if wls.scsi_vendor.is_some() || wls.scsi_product.is_some() || wls.scsi_version.is_some() {
            wl.scsi = Some(WhiteLabellingScsi::from(wls.clone()));
        }

        // Only create volume if any volume fields are present
        if wls.volume_label.is_some() || wls.uf2_model.is_some() || wls.uf2_board_id.is_some()
            || wls.redirect_name.is_some() || wls.redirect_url.is_some() {
            wl.volume = Some(WhiteLabellingVolume::from(wls.clone()));
        } else {
            wl.volume = None;
        }

        wl
    }
}

impl WhiteLabelStruct {
    /// Creates a WhiteLabelStruct from a JSON WhiteLabelling
    /// representation.
    ///
    /// Returns `Self` or a `serde_json::Error` if parsing fails.
    ///
    /// To create a WhiteLabelStruct from the values directly, use
    /// [`Self::default`] to create an empty instnace, and set the fields as
    /// required.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let wl = WhiteLabelling::from_json(json)?;
        Ok(Self::from_white_labelling(wl))
    }

    /// Creates a JSON representation of this WhiteLabelStruct.
    pub fn to_json(&self) -> Result<serde_json::Value, Error> {
        let wl = WhiteLabelling::from(self.clone());
        let result = serde_json::to_value(&wl)?;
        Ok(result)
    }

    /// Creates a WhiteLabelStruct from a WhiteLabelling instance.
    fn from_white_labelling(wl: WhiteLabelling) -> Self {
        let vendor_id = wl.usb_vid();
        let product_id = wl.usb_pid();
        let bcd_device = wl.usb_bcd();
        let language_id = wl.usb_lang_id();
        let manufacturer = wl.usb_manufacturer();
        let product = wl.usb_product();
        let serial_number = wl.usb_serial_number();
        let attr_power = wl.usb_power_attributes();
        let volume_label = wl.volume_label();
        let scsi_vendor = wl.scsi_vendor();
        let scsi_product = wl.scsi_product();
        let scsi_version = wl.scsi_version();
        let redirect_url = wl.redirect_url();
        let redirect_name = wl.redirect_name();
        let uf2_model = wl.uf2_model();
        let uf2_board_id = wl.uf2_board_id();

        let wls = Self {
            vendor_id,
            product_id,
            bcd_device,
            language_id,
            manufacturer,
            product,
            serial_number,
            attr_power,
            volume_label,
            scsi_vendor,
            scsi_product,
            scsi_version,
            redirect_url,
            redirect_name,
            uf2_model,
            uf2_board_id,
        };

        wls.validate();

        wls
    }

    fn validate(&self) {
        // Sanity checks - ideally would make these compile time
        let mut total_rows = 0;
        for ii in U16_ROWS.iter() {
            assert!(!STRDEF_ROWS.contains(ii));
            total_rows += 1;
        }
        for ii in STRDEF_ROWS.iter() {
            assert!(!U16_ROWS.contains(ii));
            total_rows += 1;
        }
        assert_eq!(
            total_rows,
            U16_ROWS.len() + STRDEF_ROWS.len(),
            "U16_ROWS and STRDEF_ROWS must be disjoint and cover all struct fields"
        );

        if let Some(v) = self.volume_label.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.scsi_vendor.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.scsi_product.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.scsi_version.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.redirect_url.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.redirect_name.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.uf2_model.as_ref() {
            assert!(v.is_ascii());
        }
        if let Some(v) = self.uf2_board_id.as_ref() {
            assert!(v.is_ascii());
        }
    }

    /// Returns the u32 value to store in the USB_BOOT_FLAGS row, as non-ECC
    /// (raw) data.
    /// This is used to indicate which fields are present in the white label
    /// data structure are valid.
    ///
    /// The same value should also be written to USB_BOOT_FLAGS_R1 and
    /// USB_BOOT_FLAGS_R2.  The bootloader uses majority voting on these three
    /// rows to determine the valid fields.
    pub fn usb_boot_flags(&self) -> u32 {
        // This function contains extraneous code by design - essentially using
        // two independent mechanisms to track the bit index for each field, to
        // reduce the likelihood of errors.

        let mut value: u32 = 0;
        let mut shift: usize = 0;

        // Index 0 - VID
        if self.vendor_id.is_some() {
            value |= 1 << 0;
            assert_eq!(shift, 0);
        }
        shift += 1;

        // Index 1 - PID
        if self.product_id.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 1);
        }
        shift += 1;

        // Index 2 - BCD
        if self.bcd_device.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 2);
        }
        shift += 1;

        // Index 3 - Lang ID
        if self.language_id.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 3);
        }
        shift += 1;

        // Index 4 - Manufacturer
        if self.manufacturer.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 4);
        }
        shift += 1;

        // Index 5 - Product
        if self.product.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 5);
        }
        shift += 1;

        // Index 6 - Serial Number
        if self.serial_number.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 6);
        }
        shift += 1;

        // Index 7 - Power/Attributes
        if self.attr_power.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 7);
        }
        shift += 1;

        // Index 8 - Volume Label
        if self.volume_label.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 8);
        }
        shift += 1;

        // Index 9 - SCSI Vendor
        if self.scsi_vendor.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 9);
        }
        shift += 1;

        // Index 10 - SCSI Product
        if self.scsi_product.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 10);
        }
        shift += 1;

        // Index 11 - SCSI Version
        if self.scsi_version.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 11);
        }
        shift += 1;

        // Index 12 - Redirect URL
        if self.redirect_url.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 12);
        }
        shift += 1;

        // Index 13 - Redirect Name
        if self.redirect_name.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 13);
        }
        shift += 1;

        // Index 14 - UF2 Module
        if self.uf2_model.is_some() {
            value |= 1 << shift;
            assert_eq!(shift, 14);
        }

        // Index 15 - UF2 Board ID
        if self.uf2_board_id.is_some() {
            value |= 1 << (shift + 1);
            assert_eq!(shift + 1, 15);
        }

        // Finally, set the WHITE_LABEL_ADDR_VALID bit
        value |= 1 << WHITE_LABEL_ADDR_VALID_BIT_NUM;

        value
    }

    /// Returns the number of OTP rows required to store this white label
    /// structure and all associated string rows.
    pub fn otp_row_count(&self) -> usize {
        // First (re-)validate
        self.validate();

        let row_count: usize = 16 // struct fields
            + self.manufacturer.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.product.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.serial_number.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.volume_label.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_vendor.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_product.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_version.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.redirect_url.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.redirect_name.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.uf2_model.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.uf2_board_id.as_ref().map_or(0, |s| s.otp_row_count()) as usize;

        // Must be at least 16 rows.
        assert!(
            row_count >= NUM_INDEX_ROWS,
            "White label structure must be at least {NUM_INDEX_ROWS} rows"
        );

        // Strictly, we can support more than 255 rows, as the last string can
        // start at or before row 255 and extend beyond it.  However, the code
        // is simplified by limiting the total size to 255 rows.
        assert!(
            row_count <= 255,
            "White label structure exceeds maximum size of 255 rows"
        );

        row_count
    }

    // Add up the number of rows all STRDEF entries will take.  This does not
    // include the STRDEF "pointers" in the struct.
    fn total_strdef_row_count(&self) -> usize {
        let total: usize = self.manufacturer.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.product.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.serial_number.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.volume_label.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_vendor.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_product.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.scsi_version.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.redirect_url.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.redirect_name.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.uf2_model.as_ref().map_or(0, |s| s.otp_row_count()) as usize
            + self.uf2_board_id.as_ref().map_or(0, |s| s.otp_row_count()) as usize;

        total
    }

    /// Returns a `Vec<u16>` containing all of the rows required to store the
    /// USB white label structure in OTP, including the row pointed to by
    /// USB_WHITE_LABEL_DATA.
    ///
    /// Rules for these rows:
    /// - They must be written contiguously.
    /// - They must be written to empty rows.  Technically, they can be written
    ///   to rows with a single set bit, and ECC will correct this.
    /// - They must be written using ECC.
    /// - Locate them in pages unreserved for Raspberry Pi use - rows 0x0c0
    ///   through 0xf3f inclusive.
    /// - By convention, the first OTP white label row is typically located at
    ///   0x100.
    ///
    /// Returns `Vec<u16>` which contains  the rows to write to OTP, starting
    /// at the location you choose, and write separately to
    /// USB_WHITE_LABEL_ADDR (0x5c).  The offset chosen for the USB white
    /// label data is often 0x100, as it's in a normally clear, unreserved
    /// area.
    pub fn to_otp_rows(&self) -> Vec<u16> {
        // First (re-)validate
        self.validate();

        // Calculate how many rows will be required.  This is not just to make
        // the vec allocation more efficient, but also to help catch errors in
        // case we miscount.
        let row_count = self.otp_row_count();

        // Create a Vec and initialize it with zeros.
        let mut rows: Vec<u16> = vec![0u16; row_count];
        let mut index = 0;
        let mut data_offset = NUM_INDEX_ROWS as u8; // first free row after struct fields
        assert!(row_count <= 255, "Row count exceeds maximum of 255");

        // Next, add each field in order, if present.
        rows[index] = self.vendor_id.unwrap_or(0);
        index += 1;

        rows[index] = self.product_id.unwrap_or(0);
        index += 1;

        rows[index] = self.bcd_device.unwrap_or(0);
        index += 1;

        rows[index] = self.language_id.unwrap_or(0);
        index += 1;

        rows[index] = self
            .manufacturer
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .product
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .serial_number
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self.attr_power.unwrap_or(0);
        index += 1;

        rows[index] = self
            .volume_label
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .scsi_vendor
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .scsi_product
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .scsi_version
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .redirect_url
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .redirect_name
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .uf2_model
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        rows[index] = self
            .uf2_board_id
            .as_ref()
            .map_or(0, |s| s.to_strdef(&mut data_offset));
        index += 1;

        // Check things are going smoothly.
        assert_eq!(
            index, NUM_INDEX_ROWS,
            "Incorrect number of struct fields written"
        );

        // Now write each string in order, if present.
        if let Some(s) = self.manufacturer.as_ref() {
            assert!(STRDEF_ROWS.contains(&4));
            write_otp_string_rows(s, &mut rows, &mut index, 4);
        }
        if let Some(s) = self.product.as_ref() {
            assert!(STRDEF_ROWS.contains(&5));
            write_otp_string_rows(s, &mut rows, &mut index, 5);
        }
        if let Some(s) = self.serial_number.as_ref() {
            assert!(STRDEF_ROWS.contains(&6));
            write_otp_string_rows(s, &mut rows, &mut index, 6);
        }
        if let Some(s) = self.volume_label.as_ref() {
            assert!(STRDEF_ROWS.contains(&8));
            write_otp_string_rows(s, &mut rows, &mut index, 8);
        }
        if let Some(s) = self.scsi_vendor.as_ref() {
            assert!(STRDEF_ROWS.contains(&9));
            write_otp_string_rows(s, &mut rows, &mut index, 9);
        }
        if let Some(s) = self.scsi_product.as_ref() {
            assert!(STRDEF_ROWS.contains(&10));
            write_otp_string_rows(s, &mut rows, &mut index, 10);
        }
        if let Some(s) = self.scsi_version.as_ref() {
            assert!(STRDEF_ROWS.contains(&11));
            write_otp_string_rows(s, &mut rows, &mut index, 11);
        }
        if let Some(s) = self.redirect_url.as_ref() {
            assert!(STRDEF_ROWS.contains(&12));
            write_otp_string_rows(s, &mut rows, &mut index, 12);
        }
        if let Some(s) = self.redirect_name.as_ref() {
            assert!(STRDEF_ROWS.contains(&13));
            write_otp_string_rows(s, &mut rows, &mut index, 13);
        }
        if let Some(s) = self.uf2_model.as_ref() {
            assert!(STRDEF_ROWS.contains(&14));
            write_otp_string_rows(s, &mut rows, &mut index, 14);
        }
        if let Some(s) = self.uf2_board_id.as_ref() {
            assert!(STRDEF_ROWS.contains(&15));
            write_otp_string_rows(s, &mut rows, &mut index, 15);
        }

        // We're done.  Check it went as expected.
        assert_eq!(index, row_count, "Incorrect total number of rows written");

        rows
    }

    /// Creates a WhiteLabelStruct from the provided OTP rows.
    ///
    /// # Args
    /// - `usb_boot_flags`: The USB_BOOT_FLAGS value stored in OTP.  This
    ///   should be read, and must be provided, as a raw (non-ECC) value.
    /// - `rows`: The OTP rows starting at the location pointed to by
    ///   USB_WHITE_LABEL_DATA.
    ///
    /// Returns `OtpParseResult` which contains any warnings generated during
    /// parsing.  This parser attempts to parse as much information as
    /// possible, and, if necessary return an incomplete WhiteLabelStruct.
    /// All u16 fields will be parsed if present, and string fields will be
    /// parsed if there are sufficient rows to do so.  However, if there are
    /// insufficient rows to parse all strings indicated by the boot flags, no
    /// strings will be parsed and a warning returned.
    ///
    /// However, it is possible for Parsing to fail and this function to
    /// return an Error if:
    /// - The rows length is less than the minimum required to store the
    ///   struct fields, so callers should ensure the slice is at least
    ///   [`NUM_INDEX_ROWS`] long.
    pub fn from_otp(usb_boot_flags: u32, rows: &[u16]) -> Result<OtpParseResult, Error> {
        // Validate we have at least the struct fields
        if rows.len() < NUM_INDEX_ROWS {
            return Err(Error::TooFewRows(NUM_INDEX_ROWS));
        }

        let mut warnings = Vec::new();

        // Check the USB boot flags
        if (usb_boot_flags & (1 << WHITE_LABEL_ADDR_VALID_BIT_NUM)) == 0 {
            warnings.push(format!(
                "USB_BOOT_FLAGS bit {WHITE_LABEL_ADDR_VALID_BIT_NUM} (WHITE_LABEL_ADDR_VALID) is not set - white label data may be invalid",
            ));
        }
        if (usb_boot_flags & (1 << DP_DM_SWAP_BIT_NUM)) != 0 {
            // This is not an error, but interesting, so report it.
            warnings.push(format!(
                "USB_BOOT_FLAGS bit {DP_DM_SWAP_BIT_NUM} (DPDM_SWAP) is set",
            ));
        }
        if (usb_boot_flags & 0xFF1F0000) != 0 {
            warnings.push(format!(
                "USB_BOOT_FLAGS has invalid bits set - ignoring these",
            ));
        }
        // Now take the bottom 16 bits only
        let usb_boot_flags = (usb_boot_flags & 0x0000FFFF) as u16;

        // Extract u16 fields using boot flags
        assert_eq!(
            NUM_U16_ROWS, 5,
            "Expected 5 u16 fields in white label struct"
        );
        let vendor_id = extract_u16_field(rows, usb_boot_flags, "usb_vendor_id", &mut warnings);
        let product_id = extract_u16_field(rows, usb_boot_flags, "usb_product_id", &mut warnings);
        let bcd_device = extract_u16_field(rows, usb_boot_flags, "usb_bcd_device", &mut warnings);
        let language_id = extract_u16_field(rows, usb_boot_flags, "usb_language_id", &mut warnings);
        let attr_power = extract_u16_field(rows, usb_boot_flags, "usb_attr_power", &mut warnings);

        // Check we have enough rows for the strings indicated by the boot flags
        let expected_total_row_count = NUM_INDEX_ROWS + {
            let mut count = 0;
            for ii in STRDEF_ROWS.iter() {
                if (usb_boot_flags & (1 << ii)) != 0 {
                    // String present - get STRDEF row
                    let strdef = rows[*ii];
                    let str_row_count = OtpString::row_count_from_strdef(strdef);
                    count += str_row_count as usize;
                }
            }
            count
        };
        if rows.len() < expected_total_row_count {
            // Not enough rows to extract all strings indicated by boot flags
            // so return early with warnings and no strings.
            warnings.push(format!(
                "OTP rows length {} is less than expected {} based on USB_BOOT_FLAGS - will not extract any strings",
                rows.len(),
                expected_total_row_count
            ));
            let wl = Self {
                vendor_id,
                product_id,
                bcd_device,
                language_id,
                manufacturer: None,
                product: None,
                serial_number: None,
                attr_power: attr_power,
                volume_label: None,
                scsi_vendor: None,
                scsi_product: None,
                scsi_version: None,
                redirect_url: None,
                redirect_name: None,
                uf2_model: None,
                uf2_board_id: None,
            };
            wl.validate();
            let result = OtpParseResult {
                white_label: wl,
                warnings,
            };
            return Ok(result);
        }

        // Extract string fields
        assert!(
            NUM_STRDEF_ROWS == 11,
            "Expected 11 STRDEF fields in white label struct"
        );
        let manufacturer = OtpString::from_otp_data(
            rows,
            usb_boot_flags,
            "usb_manufacturer",
            true,
            &mut warnings,
        )
        .expect("Manufacturer string parsing failed");
        let product =
            OtpString::from_otp_data(rows, usb_boot_flags, "usb_product", true, &mut warnings)
                .expect("Product string parsing failed");
        let serial_number = OtpString::from_otp_data(
            rows,
            usb_boot_flags,
            "usb_serial_number",
            true,
            &mut warnings,
        )
        .expect("Serial number string parsing failed");
        let volume_label =
            OtpString::from_otp_data(rows, usb_boot_flags, "volume_label", false, &mut warnings)
                .expect("Volume label string parsing failed");
        let scsi_vendor =
            OtpString::from_otp_data(rows, usb_boot_flags, "scsi_vendor", false, &mut warnings)
                .expect("SCSI vendor string parsing failed");
        let scsi_product =
            OtpString::from_otp_data(rows, usb_boot_flags, "scsi_product", false, &mut warnings)
                .expect("SCSI product string parsing failed");
        let scsi_version =
            OtpString::from_otp_data(rows, usb_boot_flags, "scsi_version", false, &mut warnings)
                .expect("SCSI version string parsing failed");
        let redirect_url =
            OtpString::from_otp_data(rows, usb_boot_flags, "redirect_url", false, &mut warnings)
                .expect("Redirect URL string parsing failed");
        let redirect_name =
            OtpString::from_otp_data(rows, usb_boot_flags, "redirect_name", false, &mut warnings)
                .expect("Redirect name string parsing failed");
        let uf2_model =
            OtpString::from_otp_data(rows, usb_boot_flags, "uf2_model", false, &mut warnings)
                .expect("UF2 model string parsing failed");
        let uf2_board_id =
            OtpString::from_otp_data(rows, usb_boot_flags, "uf2_board_id", false, &mut warnings)
                .expect("UF2 board ID string parsing failed");

        let wl = Self {
            vendor_id,
            product_id,
            bcd_device,
            language_id,
            manufacturer,
            product,
            serial_number,
            attr_power,
            volume_label,
            scsi_vendor,
            scsi_product,
            scsi_version,
            redirect_url,
            redirect_name,
            uf2_model,
            uf2_board_id,
        };

        let actual_strings_row_count = wl.total_strdef_row_count();
        let expected_row_count = NUM_INDEX_ROWS + actual_strings_row_count;
        if rows.len() != expected_row_count {
            warnings.push(format!(
                "OTP rows length {} does not match expected {} based on actual string data",
                rows.len(),
                expected_row_count
            ));
        }

        // Validate the extracted structure
        // Note: This validation can panic because if the data passed these
        // checks it means our parsing logic is wrong, not the data.
        wl.validate();

        Ok(OtpParseResult {
            white_label: wl,
            warnings,
        })
    }

    /// Similar to [`Self::from_otp`] but expects an entire ECC OTP memory
    /// dump, and extracts the white label data from the location pointed to
    /// by USB_WHITE_LABEL_DATA.
    ///
    /// Note that the USB_BOOT_FLAGS value must still be provided, as it is
    /// not possible to reliably extract it from the OTP ECC dump - as it is
    /// not stored as an ECC value.  It is expected that the caller will have
    /// done any checking of the three copies of USB_BOOT_FLAGS to determine
    /// the correct value to use.
    /// 
    /// If you want to ignore the lack of WHITE_LABEL_ADDR_VALID bit being
    /// set, and proceed to parse the white label data anyway, set
    /// `ignore_invalid_white_label_address` to true.
    ///
    /// Returns as [`Self::from_otp`].
    pub fn from_complete_otp(
        usb_boot_flags: u32,
        otp_rows: &[u16],
        ignore_invalid_white_label_address: bool,
    ) -> Result<OtpParseResult, Error> {
        if otp_rows.len() < TOTAL_OTP_ROWS {
            return Err(Error::TooFewRows(TOTAL_OTP_ROWS));
        }
        if otp_rows.len() > TOTAL_OTP_ROWS {
            return Err(Error::TooManyRows(TOTAL_OTP_ROWS));
        }

        let mut warnings = Vec::new();

        // Check the WHITE_LABEL_ADDR_VALID bit is set
        if (usb_boot_flags & (1 << WHITE_LABEL_ADDR_VALID_BIT_NUM)) == 0 {
            if !ignore_invalid_white_label_address {
               warnings.push(format!(
                    "USB_BOOT_FLAGS bit {WHITE_LABEL_ADDR_VALID_BIT_NUM} (WHITE_LABEL_ADDR_VALID) is not set - white label address may be invalid",
                ));
            } else {
                return Err(Error::InvalidWhiteLabelAddress);
            }
        }

        // Find row USB_WHITE_LABEL_DATA points to - this is where the white
        // label struct is stored.
        assert!((OTP_ROW_USB_WHITE_LABEL_DATA as usize) < TOTAL_OTP_ROWS);
        let usb_white_label_data_addr = otp_rows[OTP_ROW_USB_WHITE_LABEL_DATA as usize];
        let wl_addr = usb_white_label_data_addr as usize;
        if wl_addr >= (TOTAL_OTP_ROWS - 16) {
            return Err(Error::InvalidWhiteLabelAddressValue(
                usb_white_label_data_addr,
            ));
        }

        assert!(wl_addr + NUM_INDEX_ROWS <= TOTAL_OTP_ROWS);

        let rows = &otp_rows[wl_addr..];
        Self::from_otp(usb_boot_flags, rows)
            .map(|mut r| {
                // Add on any warnings from this function
                r.warnings.extend(warnings);
                r
            })
    }
}

// Writes the OTP rows for an OtpString into the provided rows Vec,
// starting at start_index, and updates start_index to point to the next
// free index after writing.
//
// Does pre validation to ensure that the string rows are written at the
// correct location as indicated by the strdef row at strdef_row_index.
fn write_otp_string_rows(
    otp_string: &OtpString,
    rows: &mut Vec<u16>,
    start_index: &mut usize,
    strdef_row_index: usize,
) {
    let orig_start_index = *start_index;

    // Validate
    assert_eq!(
        orig_start_index,
        OtpString::offset_from_row(rows[strdef_row_index]) as usize
    );
    assert_eq!(
        otp_string.char_count(),
        OtpString::char_count_from_row(rows[strdef_row_index])
    );
    assert_eq!(
        otp_string.is_utf16(),
        OtpString::is_utf16_from_row(rows[strdef_row_index])
    );

    // Write string rows
    let str_rows = otp_string.to_otp_rows();
    for r in str_rows {
        rows[*start_index] = r;
        *start_index += 1;
    }
}

/// Result of parsing OTP white label data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpParseResult {
    pub white_label: WhiteLabelStruct,
    pub warnings: Vec<String>,
}

impl OtpParseResult {
    /// Returns false if there were any warnings during parsing.
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Extracts a u16 field from OTP rows, checking the boot flag bit.
///
/// If the boot flag bit is clear but the row contains non-zero data,
/// a warning is added.
fn extract_u16_field(
    rows: &[u16],
    usb_boot_flags: u16,
    field_name: &str,
    warnings: &mut Vec<String>,
) -> Option<u16> {
    let index = FIELD_NAMES
        .iter()
        .position(|&name| name == field_name)
        .ok_or_else(|| format!("Invalid field name: {}", field_name))
        .unwrap(); // safe unwrap as FIELD_NAMES is static

    let bit_set = (usb_boot_flags & (1 << index)) != 0;
    let value = rows[index];

    if bit_set {
        Some(value)
    } else {
        // Boot flag bit is clear - field should be None.  This may be valid -
        // the user may have written USB white label information but not set it
        // to valid, so it is unused by the bootloader.
        if value != 0 {
            warnings.push(format!(
                "{}: boot flag bit {} clear but row contains non-zero value 0x{:04X}",
                field_name, index, value
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// Test parsing the sample JSON to OTP data and back again..
    #[test]
    fn test_sample() {
        let json = include_str!("../../json/sample-wl.json");
        let wl = WhiteLabelStruct::from_json(json);
        assert!(wl.is_ok());
        let wl = wl.unwrap();

        // Get and check the USB boot flags
        let usb_boot_flags = wl.usb_boot_flags();
        // Bit 22 set, bits 0-15 set, bits 3/7 clear
        assert_eq!(
            usb_boot_flags, 0x0040_ff77,
            "USB boot flags do not match expected value"
        );

        // Get the row data
        let otp_rows = wl.to_otp_rows();
        // 16 rows + string lengths
        assert_eq!(
            otp_rows.len(),
            75,
            "OTP row count does not match expected value"
        );

        // Now convert back again
        let new_wl = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows);
        assert!(new_wl.is_ok());

        // Check no warnings
        let new_wl = new_wl.unwrap();
        assert!(
            new_wl.is_clean(),
            "Warnings during OTP parsing: {:?}",
            new_wl.warnings
        );

        // Check the parsed white label matches the original
        assert_eq!(
            wl, new_wl.white_label,
            "Parsed white label does not match original"
        );
    }

    #[test]
    fn test_both_ways() {
        let json_files = [
            "json/sample-wl.json",
            "json/test/basic.json",
            "json/test/utf16.json",
            "json/test/complete.json",
        ];
        for file in json_files.iter() {
            let json = std::fs::read_to_string(file)
                .expect(&format!("Failed to read JSON file {}", file));
            let orig_json: serde_json::Value = serde_json::from_str(&json).expect(&format!(
                "Failed to parse JSON file {}",
                file
            ));
            let wl = WhiteLabelStruct::from_json(&json);
            assert!(wl.is_ok(), "Failed to parse JSON file {}", file);
            let wl = wl.unwrap();

            let usb_boot_flags = wl.usb_boot_flags();
            let otp_rows = wl.to_otp_rows();
            let new_wl = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows);
            assert!(new_wl.is_ok(), "Failed to parse OTP rows from file {}", file);
            let new_wl = new_wl.unwrap();
            assert!(
                new_wl.is_clean(),
                "Warnings during OTP parsing from file {}: {:?}",
                file,
                new_wl.warnings
            );
            assert_eq!(
                wl, new_wl.white_label,
                "Parsed white label does not match original for file {}",
                file
            );

            // Turn the OTP rows back into JSON and check it matches
            let wl2 = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows)
                .expect(&format!("Failed to parse OTP rows from file {}", file))
                .white_label;
            let new_json = wl2.to_json().expect(&format!(
                "Failed to convert white label to JSON for file {}",
                file
            ));
            assert_eq!(
                orig_json, new_json,
                "Re-converted JSON does not match original for file {}",
                file
            );
        }

    }

    #[test]
    fn test_utf16() {
        let json = include_str!("../../json/test/utf16.json");
        let wl = WhiteLabelStruct::from_json(json);
        assert!(wl.is_ok());
        let wl = wl.unwrap();

        // Get and check the USB boot flags
        let usb_boot_flags = wl.usb_boot_flags();
        // Bit 22 set, bits 4/5/6
        assert_eq!(
            usb_boot_flags, 0x0040_0070,
            "USB boot flags do not match expected value"
        );

        // Get the row data
        let otp_rows = wl.to_otp_rows();
        // 16 rows + string lengths
        assert_eq!(
            otp_rows.len(),
            46,
            "OTP row count does not match expected value"
        );

        // Now convert back again
        let new_wl = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows);
        assert!(new_wl.is_ok());

        // Check no warnings
        let new_wl = new_wl.unwrap();
        assert!(
            new_wl.is_clean(),
            "Warnings during OTP parsing: {:?}",
            new_wl.warnings
        );

        // Check the parsed white label matches the original
        assert_eq!(
            wl, new_wl.white_label,
            "Parsed white label does not match original"
        );
    }

    #[test]
    fn test_complete() {
        let json = include_str!("../../json/test/complete.json");
        let wl = WhiteLabelStruct::from_json(json);
        assert!(wl.is_ok());
        let wl = wl.unwrap();
        let usb_boot_flags = wl.usb_boot_flags();
        assert_eq!(
            usb_boot_flags, 0x0040_FFFF,
            "USB boot flags do not match expected value"
        );
        let otp_rows = wl.to_otp_rows();
        let new_wl = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows);
        assert!(new_wl.is_ok());
        let new_wl = new_wl.unwrap();
        assert!(
            new_wl.is_clean(),
            "Warnings during OTP parsing: {:?}",
            new_wl.warnings
        );
        assert_eq!(
            wl, new_wl.white_label,
            "Parsed white label does not match original"
        );
    }

    #[test]
    fn test_empty() {
        let json = "{}";
        let wl = WhiteLabelStruct::from_json(json);
        assert!(wl.is_ok());
        let wl = wl.unwrap();
        let usb_boot_flags = wl.usb_boot_flags();
        assert_eq!(
            usb_boot_flags, 0x0040_0000,
            "USB boot flags do not match expected value"
        );
        let otp_rows = wl.to_otp_rows();
        assert_eq!(
            otp_rows.len(),
            16,
            "OTP row count does not match expected value"
        );
        let new_wl = WhiteLabelStruct::from_otp(usb_boot_flags, &otp_rows);
        assert!(new_wl.is_ok());
        let new_wl = new_wl.unwrap();
        assert!(
            new_wl.is_clean(),
            "Warnings during OTP parsing: {:?}",
            new_wl.warnings
        );
        assert_eq!(
            wl, new_wl.white_label,
            "Parsed white label does not match original"
        );
    }

    #[test]
    fn test_bad_json() {
        let json = "{ \"usb_vendor_id\": \"not-a-number\" }";
        let wl = WhiteLabelStruct::from_json(json);
        assert!(wl.is_err(), "Expected error parsing bad JSON");
    }

    const OTP_ROWS_COMPLETE_CONFIG: [u16; 75] = [
            0x1234, 0x4678, 0x0100, 0x0409, 0x100B, 0x1608, 0x1A08, 0xFA80, 0x1E0B, 0x2408, 0x2808,
            0x2C04, 0x2E14, 0x380B, 0x3E08, 0x4211, 0x6970, 0x7265, 0x2E73, 0x6F72, 0x6B63, 0x0073,
            0x6970, 0x6F63, 0x6F2D, 0x7074, 0x3231, 0x3433, 0x6261, 0x6463, 0x4950, 0x5245, 0x2E53,
            0x4F52, 0x4B43, 0x0053, 0x6970, 0x7265, 0x7273, 0x736B, 0x6970, 0x6F63, 0x6F2D, 0x7074,
            0x3176, 0x3332, 0x7468, 0x7074, 0x3A73, 0x2F2F, 0x6970, 0x7265, 0x2E73, 0x6F72, 0x6B63,
            0x2F73, 0x6970, 0x7265, 0x2E73, 0x6F72, 0x6B63, 0x0073, 0x6970, 0x6F63, 0x6F2D, 0x7074,
            0x6970, 0x6F63, 0x6F2D, 0x7074, 0x6220, 0x616F, 0x6472, 0x6920, 0x0064,
        ];

        #[test]
    fn test_from_otp_data() {
            // From parsig json/test/complete.json
            let row_data = OTP_ROWS_COMPLETE_CONFIG;
            let usb_boot_flags = 0x0040_ffff;

        let parse_result = WhiteLabelStruct::from_otp(usb_boot_flags, &row_data);
        assert!(parse_result.is_ok(), "Expected successful parsing");
        let parse_result = parse_result.unwrap();
        assert!(
            parse_result.is_clean(),
            "Expected no warnings during parsing"
        );
        let wl = parse_result.white_label;

        // Generate expected white label from JSON and check the same
        let json = include_str!("../../json/test/complete.json");
        let expected_wl = WhiteLabelStruct::from_json(json).unwrap();
        assert_eq!(
            wl, expected_wl,
            "Parsed white label does not match expected"
        );

        // And generate the binary data again
        let regenerated_rows = wl.to_otp_rows();
        assert_eq!(
            row_data.to_vec(),
            regenerated_rows,
            "Regenerated OTP rows do not match original"
        );

        // Back to JSON schema
        let regenerated_wling = WhiteLabelling::from(wl);
        let new_json = serde_json::to_value(regenerated_wling).unwrap();
        let orig_json = serde_json::from_str::<serde_json::Value>(json).unwrap();
        assert_eq!(
            orig_json,
            new_json,
            "Regenerated JSON does not match original"
        );
    }

    #[test]
    fn test_from_otp_data_warnings() {
        // From parsig json/test/complete.json
        let row_data = OTP_ROWS_COMPLETE_CONFIG;
        let usb_boot_flags = 0x0000_ffff;  // No addr valid flag
        let parse_result = WhiteLabelStruct::from_otp(usb_boot_flags, &row_data);
        assert!(parse_result.is_ok(), "Expected successful parsing");
        let parse_result = parse_result.unwrap();
        assert!(
            !parse_result.is_clean(),
            "Expected warnings during parsing"
        );
        let warnings = parse_result.warnings;
        assert_eq!(
            warnings.len(), 1,
            "Expected one warning during parsing"
        );
        assert!(
            warnings[0].contains("WHITE_LABEL_ADDR_VALID"),
            "Expected WHITE_LABEL_ADDR_VALID warning"
        );
    }
}
