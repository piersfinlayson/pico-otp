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

use crate::whitelabel::auto::{
    WhiteLabellingDeviceAttributes, WhiteLabellingDeviceManufacturer, WhiteLabellingDeviceMaxPower,
    WhiteLabellingDeviceProduct, WhiteLabellingDeviceSerialNumber,
};
use crate::whitelabel::auto::{
    WhiteLabellingScsiProduct, WhiteLabellingScsiVendor, WhiteLabellingScsiVersion,
};
use crate::whitelabel::auto::{
    WhiteLabellingVolumeBoardId, WhiteLabellingVolumeLabel, WhiteLabellingVolumeModel,
    WhiteLabellingVolumeRedirectName, WhiteLabellingVolumeRedirectUrl,
};
use crate::whitelabel::{
    Error, OtpData, OtpString, WhiteLabelling, WhiteLabellingDevice, WhiteLabellingScsi,
    WhiteLabellingVolume,
};
use crate::whitelabel::fields::{
    Field,
    FIELDS,
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
};

// Number of rows in the white label struct that are u16 fields.
const NUM_U16_ROWS: usize = 5;
// Indices of the u16 rows in the white label struct.
const U16_ROWS: [usize; NUM_U16_ROWS] = [0, 1, 2, 3, 7];
// Number of rows in the white label struct that are STRDEF pointers.
const NUM_STRDEF_ROWS: usize = 11;
// Indices of the STRDEF rows in the white label struct.
const STRDEF_ROWS: [usize; NUM_STRDEF_ROWS] = [4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15];
// Total number of rows in the white label struct.
const NUM_INDEX_ROWS: usize = NUM_U16_ROWS + NUM_STRDEF_ROWS;
// White label address value valid bit index within the USB_BOOT_FLAGS
pub(crate) const WHITE_LABEL_ADDR_VALID_BIT_NUM: usize = 22;
// DP/DM Swap bit index within the USB_BOOT_FLAGS
const DP_DM_SWAP_BIT_NUM: usize = 23;
// Total number of rows the RP2350's OTP memory
pub(crate) const TOTAL_OTP_ROWS: usize = 4096;

/// OTP row index for USB_BOOT_FLAGS
pub const OTP_ROW_USB_BOOT_FLAGS: u16 = 0x059;
/// OTP row index for USB_BOOT_FLAGS_R1
pub const OTP_ROW_USB_BOOT_FLAGS_R1: u16 = 0x05a;
/// OTP row index for USB_BOOT_FLAGS_R2
pub const OTP_ROW_USB_BOOT_FLAGS_R2: u16 = 0x05b;
/// OTP row index for USB_WHITE_LABEL_DATA
pub const OTP_ROW_USB_WHITE_LABEL_DATA: u16 = 0x05c;

/// Start of unreserved OTP rows for white label data storage as laid out in
/// the datasheet.  Pages 0-1 are reserved for Raspberry Pi use, and page 2
/// is assumed to be reserved for bootloader use.
pub const OTP_ROW_UNRESERVED_START: u16 = 0x0c0;

/// End of unreserved OTP rows for white label data storage as laid out in the
/// datasheet.  This row itself is reserved.  Pages 61-63 are reserved for
/// Raspberry Pi use.
pub const OTP_ROW_UNRESERVED_END: u16 = 0xf40;

/// URL of the official Raspberry Pi picotool white label JSON schema,
/// supported by this crate
pub const WHITE_LABEL_SCHEMA_URL: &str = "https://raw.githubusercontent.com/piersfinlayson/pico-otp/main/json/whitelabel-schema.json";

/// Represents the USB white label structure stored in OTP.
///
/// Use [`WhiteLabelStruct::from_json`] to create an instance from a JSON
/// whitelabel representation, or use [`From<OtpData>::from`].
///
/// To create an empty instance, use [`WhiteLabelStruct::default`] and set
/// fields as required.
///
/// To create the OTP data required to store this white label structure on the
/// device, use [`From<WhiteLabelStruct>::from`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WhiteLabelStruct {
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    bcd_device: Option<u16>,
    language_id: Option<u16>,
    manufacturer: Option<OtpString>,
    product: Option<OtpString>,
    serial_number: Option<OtpString>,
    attr_power: Option<u16>,
    volume_label: Option<OtpString>,
    scsi_vendor: Option<OtpString>,
    scsi_product: Option<OtpString>,
    scsi_version: Option<OtpString>,
    redirect_url: Option<OtpString>,
    redirect_name: Option<OtpString>,
    uf2_model: Option<OtpString>,
    uf2_board_id: Option<OtpString>,
    warnings: Vec<String>,
}

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
        let manufacturer = wls.manufacturer.as_ref().map(|s| {
            WhiteLabellingDeviceManufacturer::from_str(&s.to_string())
                .expect("Invalid manufacturer string")
        });
        let product = wls.product.as_ref().map(|s| {
            WhiteLabellingDeviceProduct::from_str(&s.to_string()).expect("Invalid product string")
        });
        let serial_number = wls.serial_number.as_ref().map(|s| {
            WhiteLabellingDeviceSerialNumber::from_str(&s.to_string())
                .expect("Invalid serial number string")
        });
        let attributes = wls.attr_power.as_ref().map(|v| {
            WhiteLabellingDeviceAttributes::String(format!("{:#04x}", ((*v & 0xFF) as u8)))
        });
        let max_power = wls.attr_power.as_ref().map(|v| {
            WhiteLabellingDeviceMaxPower::String(format!("{:#04x}", (((*v & 0xFF00) >> 8) as u8)))
        });

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

impl From<WhiteLabelStruct> for WhiteLabellingScsi {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut scsi = WhiteLabellingScsi::default();

        let vendor = wls.scsi_vendor.as_ref().map(|s| {
            WhiteLabellingScsiVendor::from_str(&s.to_string()).expect("Invalid SCSI vendor string")
        });
        let product = wls.scsi_product.as_ref().map(|s| {
            WhiteLabellingScsiProduct::from_str(&s.to_string())
                .expect("Invalid SCSI product string")
        });
        let version = wls.scsi_version.as_ref().map(|s| {
            WhiteLabellingScsiVersion::from_str(&s.to_string())
                .expect("Invalid SCSI version string")
        });

        scsi.vendor = vendor;
        scsi.product = product;
        scsi.version = version;

        scsi
    }
}

impl From<WhiteLabelStruct> for WhiteLabellingVolume {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut volume = WhiteLabellingVolume::default();

        let label = wls.volume_label.as_ref().map(|s| {
            WhiteLabellingVolumeLabel::from_str(&s.to_string())
                .expect("Invalid volume label string")
        });
        let model = wls.uf2_model.as_ref().map(|s| {
            WhiteLabellingVolumeModel::from_str(&s.to_string()).expect("Invalid UF2 model string")
        });
        let board_id = wls.uf2_board_id.as_ref().map(|s| {
            WhiteLabellingVolumeBoardId::from_str(&s.to_string())
                .expect("Invalid UF2 board ID string")
        });
        let redirect_name = wls.redirect_name.as_ref().map(|s| {
            WhiteLabellingVolumeRedirectName::from_str(&s.to_string())
                .expect("Invalid redirect name string")
        });
        let redirect_url = wls.redirect_url.as_ref().map(|s| {
            WhiteLabellingVolumeRedirectUrl::from_str(&s.to_string())
                .expect("Invalid redirect URL string")
        });

        volume.label = label;
        volume.model = model;
        volume.board_id = board_id;
        volume.redirect_name = redirect_name;
        volume.redirect_url = redirect_url;

        volume
    }
}

impl From<WhiteLabelStruct> for WhiteLabelling {
    fn from(wls: WhiteLabelStruct) -> Self {
        let mut wl = WhiteLabelling::default();

        wl.schema = Some(serde_json::Value::String(
            WHITE_LABEL_SCHEMA_URL.to_string(),
        ));

        // Only create device if any device fields are present
        if wls.vendor_id.is_some()
            || wls.product_id.is_some()
            || wls.bcd_device.is_some()
            || wls.language_id.is_some()
            || wls.manufacturer.is_some()
            || wls.product.is_some()
            || wls.serial_number.is_some()
            || wls.attr_power.is_some()
        {
            wl.device = Some(WhiteLabellingDevice::from(wls.clone()));
        }

        // Only create scsi if any scsi fields are present
        if wls.scsi_vendor.is_some() || wls.scsi_product.is_some() || wls.scsi_version.is_some() {
            wl.scsi = Some(WhiteLabellingScsi::from(wls.clone()));
        }

        // Only create volume if any volume fields are present
        if wls.volume_label.is_some()
            || wls.uf2_model.is_some()
            || wls.uf2_board_id.is_some()
            || wls.redirect_name.is_some()
            || wls.redirect_url.is_some()
        {
            wl.volume = Some(WhiteLabellingVolume::from(wls.clone()));
        } else {
            wl.volume = None;
        }

        wl
    }
}

/// Converts OtpData into a WhiteLabelStruct.
impl TryFrom<&OtpData> for WhiteLabelStruct {
    type Error = Error;
    fn try_from(otp_data: &OtpData) -> Result<Self, Error> {
        let result = WhiteLabelStruct::parse_otp(
            otp_data.usb_boot_flags(),
            otp_data.rows(),
        )?;

        if result.is_clean() {
            Ok(result.white_label().clone())
        } else {
            if otp_data.strict() {
                Err(Error::OtpDataError(result.warnings().join("\n")))
            } else {
                let mut wl = result.white_label().clone();
                wl.warnings = result.warnings().clone();
                Ok(wl)
            }
        }
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

        let wls = Self::from_white_labelling(wl);

        // It should be impossible for strict OTP data generation to fail here,
        // as `from_json` should return an Error if there are any issues.
        // Therefore, we generate the OTP data in strict mode, and check that
        // succeeds.
        if let Err(e) = wls.to_otp_data_strict() {
            panic!("Internal inconsistency generating OTP data: {e}");
        }

        Ok(wls)
    }

    /// Creates a JSON representation of this WhiteLabelStruct.
    pub fn to_json(&self) -> Result<serde_json::Value, Error> {
        let wl = WhiteLabelling::from(self.clone());
        let result = serde_json::to_value(&wl)?;
        Ok(result)
    }

    /// Returns false if there were any warnings during creation.  This is
    /// guaranteed to return false if strict parsing was used for an external
    /// OTP dump, or when generated internally from JSON data.  However, it
    /// may return false if non-strict parsing of external OTP data was used.
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }

    /// Returns the warnings that were generated during creation.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    // Validates all fields and updates warnings
    fn validate_fields(&mut self) {
        self.warnings = vec![];
        for field in FIELDS {
            match field.validate(&self) {
                Ok(_) => {}
                Err(e) => {
                    self.warnings.push(e);
                }
            }
        }
    }

    // Update warnings after each set operation 
    fn update_warnings(&mut self) {
        self.validate_fields();
    }

    /// Sets the USB Vendor ID.
    pub fn set_vid(&mut self, vid: u16) {
        self.vendor_id = Some(vid);
        self.update_warnings();
    }

    /// Sets the USB Product ID.
    pub fn set_pid(&mut self, pid: u16) {
        self.product_id = Some(pid);
        self.update_warnings();
    }

    /// Sets the USB Manufacturer string.
    pub fn set_manufacturer<S: AsRef<str>>(&mut self, manufacturer: S) -> Result<(), Error>{
        self.manufacturer = Some(OtpString::try_from(manufacturer.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the USB Product string.
    pub fn set_product<S: AsRef<str>>(&mut self, product: S) -> Result<(), Error> {
        self.product = Some(OtpString::try_from(product.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the USB Serial Number string.
    pub fn set_serial_number<S: AsRef<str>>(&mut self, serial_number: S) -> Result<(), Error> {
        self.serial_number = Some(OtpString::try_from(serial_number.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the USB device BCD value (e.g. 0x0200 for version 2.00).
    pub fn set_bcd_device(&mut self, bcd: u16) {
        self.bcd_device = Some(bcd);
        self.update_warnings();
    }

    /// Sets the USB Language ID.
    pub fn set_language_id(&mut self, lang_id: u16) {
        self.language_id = Some(lang_id);
        self.update_warnings();
    }

    /// Sets the USB attributes and power.
    pub fn set_attr_power(&mut self, attr: u8, power: u8) {
        self.attr_power = Some((attr as u16) | ((power as u16) << 8));
        self.update_warnings();
    }

    /// Sets the USB Volume Label string.
    pub fn set_volume_label<S: AsRef<str>>(&mut self, volume_label: S) -> Result<(), Error> {
        self.volume_label = Some(OtpString::try_from(volume_label.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the SCSI Vendor string.
    pub fn set_scsi_vendor<S: AsRef<str>>(&mut self, scsi_vendor: S) -> Result<(), Error> {
        self.scsi_vendor = Some(OtpString::try_from(scsi_vendor.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the SCSI Product string.
    pub fn set_scsi_product<S: AsRef<str>>(&mut self, scsi_product: S) -> Result<(), Error> {
        self.scsi_product = Some(OtpString::try_from(scsi_product.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the SCSI Version string.
    pub fn set_scsi_version<S: AsRef<str>>(&mut self, scsi_version: S) -> Result<(), Error> {
        self.scsi_version = Some(OtpString::try_from(scsi_version.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the UF2 Model string.
    pub fn set_uf2_model<S: AsRef<str>>(&mut self, uf2_model: S) -> Result<(), Error> {
        self.uf2_model = Some(OtpString::try_from(uf2_model.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the UF2 Board ID string.
    pub fn set_uf2_board_id<S: AsRef<str>>(&mut self, uf2_board_id: S) -> Result<(), Error> {
        self.uf2_board_id = Some(OtpString::try_from(uf2_board_id.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the Redirect URL string.
    pub fn set_redirect_url<S: AsRef<str>>(&mut self, redirect_url: S) -> Result<(), Error> {
        self.redirect_url = Some(OtpString::try_from(redirect_url.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Sets the Redirect Name string.
    pub fn set_redirect_name<S: AsRef<str>>(&mut self, redirect_name: S) -> Result<(), Error> {
        self.redirect_name = Some(OtpString::try_from(redirect_name.as_ref())?);
        self.update_warnings();
        Ok(())
    }

    /// Returns the USB Vendor ID, if set.
    pub fn vid(&self) -> Option<u16> {
        self.vendor_id
    }

    /// Returns the USB Product ID, if set.
    pub fn pid(&self) -> Option<u16> {
        self.product_id
    }

    /// Returns a reference to the USB Manufacturer string, if set.
    pub fn manufacturer(&self) -> Option<&String> {
        self.manufacturer.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the USB Product string, if set.
    pub fn product(&self) -> Option<&String> {
        self.product.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the USB Serial Number string, if set.
    pub fn serial_number(&self) -> Option<&String> {
        self.serial_number.as_ref().map(|s| s.string())
    }

    /// Returns the USB BCD value, if set.
    pub fn bcd_device(&self) -> Option<u16> {
        self.bcd_device
    }

    /// Returns the USB Language ID, if set.
    pub fn language_id(&self) -> Option<u16> {
        self.language_id
    }

    /// Returns the USB attributes and power, if set.
    pub fn attr_power(&self) -> Option<u16> {
        self.attr_power
    }

    /// Returns power attributes if set.
    pub fn power(&self) -> Option<u8> {
        self.attr_power.map(|v| ((v & 0xFF00) >> 8) as u8)
    }

    /// Returns the USB attributes and power, if set.
    pub fn attributes(&self) -> Option<u8> {
        self.attr_power.map(|v| (v & 0x00FF) as u8)
    }

    /// Returns a reference to the USB Volume Label string, if set.
    pub fn volume_label(&self) -> Option<&String> {
        self.volume_label.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the SCSI Vendor string, if set.
    pub fn scsi_vendor(&self) -> Option<&String> {
        self.scsi_vendor.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the SCSI Product string, if set.
    pub fn scsi_product(&self) -> Option<&String> {
        self.scsi_product.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the SCSI Version string, if set.
    pub fn scsi_version(&self) -> Option<&String> {
        self.scsi_version.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the UF2 Model string, if set.
    pub fn uf2_model(&self) -> Option<&String> {
        self.uf2_model.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the UF2 Board ID string, if set.
    pub fn uf2_board_id(&self) -> Option<&String> {
        self.uf2_board_id.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the Redirect URL string, if set.
    pub fn redirect_url(&self) -> Option<&String> {
        self.redirect_url.as_ref().map(|s| s.string())
    }

    /// Returns a reference to the Redirect Name string, if set.
    pub fn redirect_name(&self) -> Option<&String> {
        self.redirect_name.as_ref().map(|s| s.string())
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

        let mut wls = Self {
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
            warnings: vec![],
        };

        wls.validate();
        wls.validate_fields();

        if !wls.is_clean() {
            panic!(
                "Invalid WhiteLabelStruct created from WhiteLabelling: {:?}",
                wls.warnings()
            );
        }
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

    /// Creates the OTP data required to store this white label structure,
    /// returning an error if there were any issues found within the white
    /// label structure.
    ///
    /// The use of this function is strongly recommended over
    /// [`to_otp_data_loose`](`Self::to_otp_data_loose`) where you are
    /// generating the data to be written to a real device.
    ///
    /// Returns:
    /// - `Ok(OtpData)` if the white label structure is valid.
    /// - `Err(Error::InvalidWhiteLabelData)` if the white label structure is
    ///   invalid or inconsistent.
    pub fn to_otp_data_strict(&self) -> Result<OtpData, Error> {
        if !self.is_clean() {
            let warnings = self.warnings().join("\n");
            return Err(Error::InvalidWhiteLabelData(warnings));
        }
        Ok(self.create_otp_data(true))
    }

    /// Creates the OTP data required to store this white label structure.
    /// Succeeds if there are any warnings present in the white label
    /// structure.
    ///
    /// The use of [`to_otp_data_strict`](`Self::to_otp_data_strict`) is
    /// strongly recommended over this function where you are generating the
    /// data to be written to a real device.
    ///
    /// Returns:
    /// - `OtpData` containing the OTP data rows.
    pub fn to_otp_data_loose(&self) -> OtpData {
        self.create_otp_data(false)
    }

    fn create_otp_data(&self, strict: bool) -> OtpData {
        let rows = self.to_otp_rows();
        let usb_boot_flags = self.usb_boot_flags();
        OtpData::new(usb_boot_flags, rows, strict)
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
    pub(crate) fn to_otp_rows(&self) -> Vec<u16> {
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

    /// Return a WhiteLabelStruct from the provided OTP rows.
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
    pub(crate) fn parse_otp(usb_boot_flags: u32, rows: &[u16]) -> Result<OtpParseResult, Error> {
        // Validate we have at least the struct fields
        if rows.len() < NUM_INDEX_ROWS {
            return Err(Error::InternalInconsistency(format!(
                "Too few rows provided"
            )));
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
        let vendor_id = extract_u16_field(rows, usb_boot_flags, &FIELD_USB_VENDOR_ID, &mut warnings);
        let product_id = extract_u16_field(rows, usb_boot_flags, &FIELD_USB_PRODUCT_ID, &mut warnings);
        let bcd_device = extract_u16_field(rows, usb_boot_flags, &FIELD_USB_BCD_DEVICE, &mut warnings);
        let language_id = extract_u16_field(rows, usb_boot_flags, &FIELD_USB_LANGUAGE_ID, &mut warnings);
        let attr_power = extract_u16_field(rows, usb_boot_flags, &FIELD_USB_ATTR_POWER, &mut warnings);

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
                attr_power,
                volume_label: None,
                scsi_vendor: None,
                scsi_product: None,
                scsi_version: None,
                redirect_url: None,
                redirect_name: None,
                uf2_model: None,
                uf2_board_id: None,
                warnings: warnings.clone(),
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
            &FIELD_USB_MANUFACTURER,
            true,
            &mut warnings,
        )
        .expect("Manufacturer string parsing failed");
        let product =
            OtpString::from_otp_data(
                rows,
                usb_boot_flags,
                &FIELD_USB_PRODUCT,
                true,
                &mut warnings,
            )
            .expect("Product string parsing failed");
        let serial_number = OtpString::from_otp_data(
            rows,
            usb_boot_flags,
            &FIELD_USB_SERIAL_NUMBER,
            true,
            &mut warnings,
        )
        .expect("Serial number string parsing failed");
        let volume_label =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_VOLUME_LABEL, false, &mut warnings)
                .expect("Volume label string parsing failed");
        let scsi_vendor =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_SCSI_VENDOR, false, &mut warnings)
                .expect("SCSI vendor string parsing failed");
        let scsi_product =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_SCSI_PRODUCT, false, &mut warnings)
                .expect("SCSI product string parsing failed");
        let scsi_version =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_SCSI_VERSION, false, &mut warnings)
                .expect("SCSI version string parsing failed");
        let redirect_url =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_REDIRECT_URL, false, &mut warnings)
                .expect("Redirect URL string parsing failed");
        let redirect_name =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_REDIRECT_NAME, false, &mut warnings)
                .expect("Redirect name string parsing failed");
        let uf2_model =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_UF2_MODEL, false, &mut warnings)
                .expect("UF2 model string parsing failed");
        let uf2_board_id =
            OtpString::from_otp_data(rows, usb_boot_flags, &FIELD_UF2_BOARD_ID, false, &mut warnings)
                .expect("UF2 board ID string parsing failed");

        let mut wl = Self {
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
            warnings: vec![],  // Set below
        };

        let actual_strings_row_count = wl.total_strdef_row_count();
        let expected_row_count = NUM_INDEX_ROWS + actual_strings_row_count;

        // Allow too much data, but not too little
        if rows.len() < expected_row_count {
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

        wl.warnings = warnings.clone();

        Ok(OtpParseResult {
            white_label: wl,
            warnings,
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
pub(crate) struct OtpParseResult {
    white_label: WhiteLabelStruct,
    warnings: Vec<String>,
}

impl OtpParseResult {
    /// Returns false if there were any warnings during parsing.
    pub(crate) fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }

    /// Returns the [`WhiteLabelStruct`].
    pub(crate) fn white_label(&self) -> &WhiteLabelStruct {
        &self.white_label
    }

    /// Returns any warnings generated during parsing.
    pub(crate) fn warnings(&self) -> &Vec<String> {
        &self.warnings
    }
}

/// Extracts a u16 field from OTP rows, checking the boot flag bit.
///
/// If the boot flag bit is clear but the row contains non-zero data,
/// a warning is added.
fn extract_u16_field(
    rows: &[u16],
    usb_boot_flags: u16,
    field: &Field,
    warnings: &mut Vec<String>,
) -> Option<u16> {
    let index = field.index();

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
                field.name(), index, value
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
        let new_wl = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows);
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
            let json =
                std::fs::read_to_string(file).expect(&format!("Failed to read JSON file {}", file));
            let orig_json: serde_json::Value =
                serde_json::from_str(&json).expect(&format!("Failed to parse JSON file {}", file));
            let wl = WhiteLabelStruct::from_json(&json);
            assert!(wl.is_ok(), "Failed to parse JSON file {}", file);
            let wl = wl.unwrap();

            let usb_boot_flags = wl.usb_boot_flags();
            let otp_rows = wl.to_otp_rows();
            let new_wl = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows);
            assert!(
                new_wl.is_ok(),
                "Failed to parse OTP rows from file {}",
                file
            );
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
            let wl2 = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows)
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
        let new_wl = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows);
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
        let new_wl = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows);
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
        let new_wl = WhiteLabelStruct::parse_otp(usb_boot_flags, &otp_rows);
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

        let parse_result = WhiteLabelStruct::parse_otp(usb_boot_flags, &row_data);
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
            orig_json, new_json,
            "Regenerated JSON does not match original"
        );
    }

    #[test]
    fn test_from_otp_data_warnings() {
        // From parsig json/test/complete.json
        let row_data = OTP_ROWS_COMPLETE_CONFIG;
        let usb_boot_flags = 0x0000_ffff; // No addr valid flag
        let parse_result = WhiteLabelStruct::parse_otp(usb_boot_flags, &row_data);
        assert!(parse_result.is_ok(), "Expected successful parsing");
        let parse_result = parse_result.unwrap();
        assert!(!parse_result.is_clean(), "Expected warnings during parsing");
        let warnings = parse_result.warnings;
        assert_eq!(warnings.len(), 1, "Expected one warning during parsing");
        assert!(
            warnings[0].contains("WHITE_LABEL_ADDR_VALID"),
            "Expected WHITE_LABEL_ADDR_VALID warning"
        );
    }
}
