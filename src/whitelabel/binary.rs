// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Contains the OtpData object

use alloc::string::ToString;
use alloc::vec::Vec;

use crate::WhiteLabelStruct;
use crate::whitelabel::Error;
use crate::whitelabel::{
    OTP_ROW_UNRESERVED_END, OTP_ROW_UNRESERVED_START, OTP_ROW_USB_BOOT_FLAGS,
    OTP_ROW_USB_BOOT_FLAGS_R1, OTP_ROW_USB_BOOT_FLAGS_R2, OTP_ROW_USB_WHITE_LABEL_DATA,
};
use crate::whitelabel::top::{
    TOTAL_OTP_ROWS, WHITE_LABEL_ADDR_VALID_BIT_NUM,
};

// We assume a minimum of 256 rows (4 pages) being available for white label
// data storage.
// White label data could use much less than this in a minimal configuration.
// If only u16 fields are white labelled, only 16 bytes is required.  However,
// for parsing purposes we assume a minimum of 256 rows (4 pages) being
// available.

// strictly, white label data could use more than this - a final string
// could start at byte 255, and be a maximum of 127 ASCII characters long,
// being 64 rows log, thus taking a total of 319 rows.
const MIN_REQD_OTP_WHITE_LABEL_ROWS: usize = 256;

// Maximum amount of white label data possible.  Due to the format used,
// strings can only start at up to byte 255 from the start of the white label
// data.  As the final string could start at byte 255 and be a maximum of
// 127 ASCII characters long, being 64 rows long), the maximum possible
// white label data size is thus 256+63 = 319 rows.
const MAX_OTP_WHITE_LABEL_ROWS: usize = 319;

// Maximum valid WHITE_LABEL_ADDR value (row index) for white label data
const MAX_WHITELABEL_ADDR: u16 =
    (OTP_ROW_UNRESERVED_END as usize - MIN_REQD_OTP_WHITE_LABEL_ROWS) as u16;

/// Used to hold the OTP data for an RP2350's USB white label configuration.
/// This is used both by the generation code, to hold the generated OTP row
/// data, and by the parsing code, to hold the extracted OTP row data.
#[derive(Debug, Clone, PartialEq)]
pub struct OtpData {
    // The value required to be written to the
    // [`OTP_ROW_USB_BOOT_FLAGS`](super::OTP_ROW_USB_BOOT_FLAGS),
    // [`OTP_ROW_USB_BOOT_FLAGS_R1`](super::OTP_ROW_USB_BOOT_FLAGS_R1) and
    // [`OTP_ROW_USB_BOOT_FLAGS_R2`](super::OTP_ROW_USB_BOOT_FLAGS_R2) rows
    // to enable this white label configuration.
    usb_boot_flags: u32,

    // The OTP rows containing the white label data, from the struct onwards.
    rows: Vec<u16>,

    // Whether strict checking was enabled when parsing the OTP data.
    strict: bool,
}

/// Converts a WhiteLabelStruct into OtpData.  Uses strict checking - will
/// return an error if any inconsistencies are found.
impl TryFrom<WhiteLabelStruct> for OtpData {
    type Error = Error;

    fn try_from(wls: WhiteLabelStruct) -> Result<Self, Self::Error> {
        wls.to_otp_data_strict()
    }
}

/// Converts a reference to a WhiteLabelStruct into OtpData.  Uses strict
/// checking - will return an error if any inconsistencies are found.
impl TryFrom<&WhiteLabelStruct> for OtpData {
    type Error = Error;

    fn try_from(wls: &WhiteLabelStruct) -> Result<Self, Self::Error> {
        wls.to_otp_data_strict()
    }
}

impl OtpData {
    /// Creates a new OtpData object.
    /// 
    /// This is provided for convenience, but you may prefer to use one of
    /// [`from_json`](`Self::from_json`),
    /// [`from_full_otp_data`](`Self::from_full_otp_data`) or
    /// [`TryFrom<WhiteLabelStruct>`](`Self::try_from`) instead.
    ///
    /// Args:
    /// - `usb_boot_flags`: The USB boot flags value.
    /// - `rows`: The OTP rows containing the white label data, starting from
    ///   first row of the white label data struct.
    ///
    /// Returns:
    /// - `OtpData`: The created OtpData object.
    pub fn new(usb_boot_flags: u32, rows: Vec<u16>, strict: bool) -> Self {
        OtpData {
            usb_boot_flags,
            rows,
            strict,
        }
    }

    /// Creates white label OTP data directly from a JSON string.
    ///
    /// Can be used to skip the creation of the WhiteLabelStruct where that
    /// isn't required.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let wls = WhiteLabelStruct::from_json(json)?;

        match wls.to_otp_data_strict() {
            Ok(otp_data) => Ok(otp_data),
            Err(e) => {
                panic!("Internal inconsistency generating OTP data: {e}");
            }
        }
    }

    /// Returns a JSON representation of this OTP data.
    pub fn to_json(&self) -> Result<serde_json::Value, Error> {
        WhiteLabelStruct::try_from(self)?
            .to_json()
    }

    /// Creates an OtpData object from a complete OTP dump, consisting of both
    /// ECC and non-ECC data) and extracts the USB white label specific OTP
    /// data, returning a [`OtpData`].
    ///
    /// Args:
    /// - `ecc_data`: A reference to an array of 4096 u32 words containing
    ///   the ECC OTP data dump.
    /// - `non_ecc_data`: A reference to an array of 4096 u32 words containing
    ///   the non-ECC OTP data dump.
    /// - `strict`: If true, performs strict checking on the data, and will
    ///   return an error if any inconsistencies are found.  If false,
    ///   attempts to recover from inconsistencies.
    ///
    /// The types of consistencies enforced when `strict` is true are:
    /// - All three copies of the USB boot flags must match.
    /// - The WHITE_LABEL_ADDR must point to a non-reserved location in OTP
    ///   memory, with at least 256 rows available for the white label data.
    ///
    /// Returns:
    /// - `Ok(OtpData)`: The extracted OTP data.
    /// - `Err(Error)`: An error occurred while parsing the OTP data.
    pub fn from_full_otp_data(
        non_ecc_data: &[u32; TOTAL_OTP_ROWS],
        ecc_data: &[u16; TOTAL_OTP_ROWS],
        strict: bool,
    ) -> Result<Self, Error> {
        // Extract the 3 copies of the USB boot flags from the non-ECC OTP
        // data and verify they match.
        let usb_boot_flags = non_ecc_data[OTP_ROW_USB_BOOT_FLAGS as usize];
        let usb_boot_flags_r1 = non_ecc_data[OTP_ROW_USB_BOOT_FLAGS_R1 as usize];
        let usb_boot_flags_r2 = non_ecc_data[OTP_ROW_USB_BOOT_FLAGS_R2 as usize];
        let master_usb_boot_flags = if strict {
            // All three copies of the USB boot flags must match.
            if usb_boot_flags != usb_boot_flags_r1 || usb_boot_flags != usb_boot_flags_r2 {
                return Err(Error::NonMatchingUsbBootFlags);
            }
            usb_boot_flags
        } else {
            // Determine the master copy - at least two out of three must match.
            if usb_boot_flags == usb_boot_flags_r1 || usb_boot_flags == usb_boot_flags_r2 {
                usb_boot_flags
            } else if usb_boot_flags_r1 == usb_boot_flags_r2 {
                usb_boot_flags_r1
            } else {
                return Err(Error::NonMatchingUsbBootFlags);
            }
        };

        // Extract the white label data from the ECC OTP data.
        let white_label_addr = ecc_data[OTP_ROW_USB_WHITE_LABEL_DATA as usize];
        if strict {
            // Check the white label address is not in a reserved region.
            if white_label_addr < OTP_ROW_UNRESERVED_START || white_label_addr > MAX_WHITELABEL_ADDR
            {
                return Err(Error::InvalidWhiteLabelAddressValue(
                    white_label_addr as u16,
                ));
            }
        }

        // Store off the maximum required amount of white label data.  There
        // is an inconsistency here with the test above - we will copy the
        // theoretical maximum, not the minimum required amount.  This could
        // result in copying some reserved data, but shouldn't go off the end
        // of the provided OTP ECC data dump.
        assert!((white_label_addr as usize) + MAX_OTP_WHITE_LABEL_ROWS <= TOTAL_OTP_ROWS);
        let rows = Vec::from(
            &ecc_data
                [white_label_addr as usize..(white_label_addr as usize) + MAX_OTP_WHITE_LABEL_ROWS],
        );

        Self::from_white_label_data(master_usb_boot_flags, &rows, strict)
    }

    /// Creates an OtpData object from a slice of OTP ECC row data only, plus
    /// the USB boot flags.
    ///
    /// Args:
    /// - `usb_boot_flags`: The USB boot flags value.
    /// - `ecc_rows`: A slice of u16 containing the OTP ECC row data.
    /// - `strict`: If true, indicates strict checking is to be used when
    ///   parsing.  See [`from_full_otp_data`](`Self::from_full_otp_data`)
    ///   for details.
    ///
    /// Returns:
    /// - `Ok(OtpData)`: The created OtpData object.
    /// - `Err(Error)`: An error occurred while parsing the OTP data.
    pub fn from_white_label_data(
        usb_boot_flags: u32,
        ecc_rows: &[u16],
        strict: bool,
    ) -> Result<Self, Error> {
        if strict {
            if usb_boot_flags & (1 << WHITE_LABEL_ADDR_VALID_BIT_NUM) == 0 {
                return Err(Error::OtpDataError(
                    "WHITE_LABEL_ADDR_VALID bit not set in USB boot flags".to_string(),
                ));
            }
        }

        Ok(OtpData::new(usb_boot_flags, Vec::from(ecc_rows), strict))
    }

    /// Returns the USB boot flags value required to enable this white label
    /// configuration.
    pub fn usb_boot_flags(&self) -> u32 {
        self.usb_boot_flags
    }

    /// Returns a reference to the OTP rows containing the white label data.
    pub fn rows(&self) -> &Vec<u16> {
        &self.rows
    }

    /// Returns a vector of bytes representing the OTP ECC data in little-
    /// endian format, ready for writing to the OTP memory.
    pub fn to_le_ecc_bytes(&self) -> Vec<u8> {
        self.rows
            .iter()
            .flat_map(|row| row.to_le_bytes())
            .collect::<Vec<u8>>()
    }

    /// Returns whether strict checking was enabled when parsing the OTP data.
    pub fn strict(&self) -> bool {
        self.strict
    }
}
