// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Contains the OtpData object

use alloc::string::ToString;
use alloc::vec::Vec;

use crate::WhiteLabelStruct;
use crate::whitelabel::Error;
use crate::whitelabel::top::{NUM_INDEX_ROWS, TOTAL_OTP_ROWS, WHITE_LABEL_ADDR_VALID_BIT_NUM};
use crate::whitelabel::{
    OTP_ROW_UNRESERVED_END, OTP_ROW_UNRESERVED_START, OTP_ROW_USB_BOOT_FLAGS,
    OTP_ROW_USB_BOOT_FLAGS_R1, OTP_ROW_USB_BOOT_FLAGS_R2, OTP_ROW_USB_WHITE_LABEL_DATA,
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
        WhiteLabelStruct::try_from(self)?.to_json()
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
    /// When `strict` is false, WHITE_LABEL_ADDR must still leave room for the
    /// 16 rows of the white label struct before the end of OTP.
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
            if !(OTP_ROW_UNRESERVED_START..=MAX_WHITELABEL_ADDR).contains(&white_label_addr) {
                return Err(Error::InvalidWhiteLabelAddressValue(white_label_addr));
            }
        }

        // Store off the maximum required amount of white label data.  There
        // is an inconsistency here with the test above - we will copy the
        // theoretical maximum, not the minimum required amount.  This could
        // result in copying some reserved data.
        //
        // Without strict checking the address can be too near the end of OTP
        // for the maximum so copy up to the end instead.  The 16 rows of the
        // white label struct must still fit.
        let start = white_label_addr as usize;
        if start + NUM_INDEX_ROWS > TOTAL_OTP_ROWS {
            return Err(Error::InvalidWhiteLabelAddressValue(white_label_addr));
        }
        let end = TOTAL_OTP_ROWS.min(start + MAX_OTP_WHITE_LABEL_ROWS);
        let rows = Vec::from(&ecc_data[start..end]);

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
        if strict && usb_boot_flags & (1 << WHITE_LABEL_ADDR_VALID_BIT_NUM) == 0 {
            return Err(Error::OtpDataError(
                "WHITE_LABEL_ADDR_VALID bit not set in USB boot flags".to_string(),
            ));
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

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::whitelabel::fields::FIELDS;
    use alloc::vec;

    // Returns a full non-ECC OTP dump holding the three USB boot flags
    // copies.
    fn non_ecc_data(copies: [u32; 3]) -> [u32; TOTAL_OTP_ROWS] {
        let mut data = [0; TOTAL_OTP_ROWS];
        data[OTP_ROW_USB_BOOT_FLAGS as usize] = copies[0];
        data[OTP_ROW_USB_BOOT_FLAGS_R1 as usize] = copies[1];
        data[OTP_ROW_USB_BOOT_FLAGS_R2 as usize] = copies[2];
        data
    }

    // Returns a full ECC OTP dump holding the white label rows at `addr`.
    // Rows past the end of OTP are left out.
    fn ecc_data(addr: u16, rows: &[u16]) -> [u16; TOTAL_OTP_ROWS] {
        let mut data = [0; TOTAL_OTP_ROWS];
        let start = TOTAL_OTP_ROWS.min(addr as usize);
        let len = rows.len().min(TOTAL_OTP_ROWS - start);
        data[start..start + len].copy_from_slice(&rows[..len]);
        data[OTP_ROW_USB_WHITE_LABEL_DATA as usize] = addr;
        data
    }

    // Without strict checking the white label address can be too near the
    // end of OTP for the maximum amount of white label data.
    #[test]
    fn test_full_otp_white_label_near_end() {
        // A white label holding only the VID
        let flags = (1 << WHITE_LABEL_ADDR_VALID_BIT_NUM) | 1;
        let non_ecc = non_ecc_data([flags; 3]);
        let mut rows = vec![0; NUM_INDEX_ROWS];
        rows[0] = 0x1234;

        // Room for the struct
        let near_end = [
            (TOTAL_OTP_ROWS - MAX_OTP_WHITE_LABEL_ROWS + 1) as u16,
            0xF00,
            (TOTAL_OTP_ROWS - NUM_INDEX_ROWS) as u16,
        ];
        for addr in near_end {
            let ecc = ecc_data(addr, &rows);
            assert!(matches!(
                OtpData::from_full_otp_data(&non_ecc, &ecc, true),
                Err(Error::InvalidWhiteLabelAddressValue(a)) if a == addr
            ));
            let otp_data = OtpData::from_full_otp_data(&non_ecc, &ecc, false).unwrap();
            assert_eq!(otp_data.rows().len(), TOTAL_OTP_ROWS - addr as usize);
            let wls = WhiteLabelStruct::try_from(&otp_data).unwrap();
            assert_eq!(wls.vid(), Some(0x1234));
        }

        // Not enough room for the struct
        let past_end = [
            (TOTAL_OTP_ROWS - NUM_INDEX_ROWS + 1) as u16,
            TOTAL_OTP_ROWS as u16,
            u16::MAX,
        ];
        for addr in past_end {
            let ecc = ecc_data(addr, &rows);
            for strict in [true, false] {
                assert!(matches!(
                    OtpData::from_full_otp_data(&non_ecc, &ecc, strict),
                    Err(Error::InvalidWhiteLabelAddressValue(a)) if a == addr
                ));
            }
        }
    }

    // splitmix64 so every run tests the same rows
    struct Rng(u64);

    impl Rng {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }

        // Returns a value from 0 to n - 1
        fn below(&mut self, n: usize) -> usize {
            (self.next_u64() % n as u64) as usize
        }

        fn u16(&mut self) -> u16 {
            self.next_u64() as u16
        }

        fn chance(&mut self, percent: usize) -> bool {
            self.below(100) < percent
        }
    }

    // Returns a row of string data.  Each kind of row is for a different
    // check in string decoding.
    fn string_row(rng: &mut Rng, kind: usize, position: usize) -> u16 {
        let ascii = 0x20 + rng.below(0x5F) as u16;
        match kind {
            // Two ASCII characters that form one character above 0x7F as
            // UTF-16
            0 => ascii | ((0x20 + rng.below(0x5F) as u16) << 8),
            // One ASCII character as UTF-16
            1 => ascii,
            // "é" as UTF-8
            2 => 0xA9C3,
            // A character that takes two bytes as UTF-8
            3 => 0x80 + rng.below(0x780) as u16,
            // A character that takes three bytes as UTF-8
            4 => 0x800 + rng.below(0xD000) as u16,
            // Surrogate pairs that are sometimes broken
            5 if rng.chance(5) => rng.u16(),
            5 if position.is_multiple_of(2) => 0xD83D,
            5 => 0xDE00,
            // Mostly ASCII characters as UTF-16 with one "é" in ten.  Strings
            // of up to about 115 characters stay within 127 bytes as UTF-8.
            // Longer ones mostly don't.
            6 if rng.chance(90) => ascii,
            6 => 0xE9,
            _ => rng.u16(),
        }
    }

    // Returns USB boot flags and white label rows.  Most sets hold STRDEFs
    // that point inside the rows at strings of every kind above.  The rest
    // are noise.
    fn random_white_label(rng: &mut Rng) -> (u32, Vec<u16>) {
        if rng.chance(10) {
            let len = rng.below(400);
            let rows = (0..len).map(|_| rng.u16()).collect();
            return (rng.next_u64() as u32, rows);
        }

        let len = if rng.chance(5) {
            rng.below(NUM_INDEX_ROWS)
        } else {
            NUM_INDEX_ROWS + rng.below(MAX_OTP_WHITE_LABEL_ROWS)
        };
        let mut rows: Vec<u16> = (0..len)
            .map(|_| if rng.chance(50) { 0 } else { rng.u16() })
            .collect();

        for field in FIELDS.iter().filter(|field| field.is_string()) {
            let index = field.index();
            if index >= len || rng.chance(40) {
                continue;
            }

            let utf16 = rng.chance(50);
            let char_count = match rng.below(4) {
                0 => rng.below(128),
                1 => 127,
                _ => rng.below(40),
            };
            let offset = if len > NUM_INDEX_ROWS && rng.chance(90) {
                NUM_INDEX_ROWS + rng.below(len.min(256) - NUM_INDEX_ROWS)
            } else {
                rng.below(256)
            };
            let encoding = if utf16 { 0x80 } else { 0 };
            rows[index] = ((offset as u16) << 8) | encoding | char_count as u16;

            let row_count = if utf16 {
                char_count
            } else {
                char_count.div_ceil(2)
            };
            let kind = rng.below(8);
            for position in 0..row_count {
                let Some(row) = rows.get_mut(offset + position) else {
                    break;
                };
                *row = string_row(rng, kind, position);
            }
        }

        // Set mostly the white label bits and sometimes others
        let mut flags = if rng.chance(30) {
            0xFFFF
        } else {
            rng.u16() as u32
        };
        if rng.chance(80) {
            flags |= 1 << WHITE_LABEL_ADDR_VALID_BIT_NUM;
        }
        if rng.chance(20) {
            flags |= rng.next_u64() as u32 & 0xFFFF_0000;
        }
        (flags, rows)
    }

    // Returns a white label address that is often at or next to a limit
    fn random_address(rng: &mut Rng) -> u16 {
        let limits = [
            OTP_ROW_UNRESERVED_START - 1,
            OTP_ROW_UNRESERVED_START,
            MAX_WHITELABEL_ADDR,
            MAX_WHITELABEL_ADDR + 1,
            (TOTAL_OTP_ROWS - MAX_OTP_WHITE_LABEL_ROWS) as u16,
            (TOTAL_OTP_ROWS - MAX_OTP_WHITE_LABEL_ROWS + 1) as u16,
            (TOTAL_OTP_ROWS - NUM_INDEX_ROWS) as u16,
            (TOTAL_OTP_ROWS - NUM_INDEX_ROWS + 1) as u16,
            TOTAL_OTP_ROWS as u16,
            u16::MAX,
        ];
        match rng.below(3) {
            0 => limits[rng.below(limits.len())],
            1 => rng.below(TOTAL_OTP_ROWS) as u16,
            _ => rng.u16(),
        }
    }

    // Calls everything a caller can on OTP data read from a device.  A
    // struct that decodes must convert to JSON.  Strict decoding must not
    // warn.
    fn use_otp_data(otp_data: &OtpData) {
        let _ = (
            otp_data.usb_boot_flags(),
            otp_data.rows(),
            otp_data.strict(),
        );
        let _ = otp_data.to_le_ecc_bytes();
        let _ = otp_data.to_json();
        let Ok(wls) = WhiteLabelStruct::try_from(otp_data) else {
            return;
        };
        let _ = (wls.vid(), wls.pid(), wls.bcd_device(), wls.language_id());
        let _ = (wls.attr_power(), wls.power(), wls.attributes());
        let _ = (wls.manufacturer(), wls.product(), wls.serial_number());
        let _ = (wls.volume_label(), wls.scsi_vendor(), wls.scsi_product());
        let _ = (wls.scsi_version(), wls.redirect_url(), wls.redirect_name());
        let _ = (wls.uf2_model(), wls.uf2_board_id(), wls.usb_boot_flags());
        assert!(wls.is_clean() || !otp_data.strict());
        assert!(wls.to_json().is_ok());
    }

    // Decodes the rows as white label rows and at `addr` in a full OTP dump
    // with and without strict checking.
    fn decode_everything(flags: u32, rows: &[u16], addr: u16, copies: [u32; 3]) {
        let non_ecc = non_ecc_data(copies);
        let ecc = ecc_data(addr, rows);
        for strict in [false, true] {
            use_otp_data(&OtpData::new(flags, rows.to_vec(), strict));
            if let Ok(otp_data) = OtpData::from_white_label_data(flags, rows, strict) {
                use_otp_data(&otp_data);
            }
            if let Ok(otp_data) = OtpData::from_full_otp_data(&non_ecc, &ecc, strict) {
                use_otp_data(&otp_data);
            }
        }
    }

    // Checks that decoding 10,000 pseudo-random sets of rows and boot flags
    // doesn't panic.
    #[test]
    fn test_decode_random_rows() {
        let mut rng = Rng(0x7069_636F_2D6F_7470);
        for set in 0..10_000 {
            let (flags, rows) = random_white_label(&mut rng);
            let addr = random_address(&mut rng);

            // The three boot flags copies mostly agree
            let mut copies = [flags; 3];
            for copy in copies.iter_mut() {
                if rng.chance(10) {
                    *copy = rng.next_u64() as u32;
                }
            }

            let result = std::panic::catch_unwind(|| decode_everything(flags, &rows, addr, copies));
            assert!(
                result.is_ok(),
                "Set {set} panicked: flags {flags:08x}, address {addr:04x}, copies {copies:08x?}, rows {rows:04x?}"
            );
        }
    }
}
