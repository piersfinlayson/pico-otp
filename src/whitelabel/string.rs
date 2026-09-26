// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Handles the OTP STRDEF object type

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::whitelabel::Error;
use crate::whitelabel::fields::{Field, MAX_STRING_LENGTH, NUM_FIELDS};

/// Represents a string to be stored in OTP, using the STRDEF encoding defined
/// in the RP2350 datasheet at OTP_DATA:USB_WHITE_LABEL_ADDR Register, section
/// 13.10.
///
/// The object supports both ASCII and UTF-16 strings.  However, only the USB
/// manufacturer, product and serial number strings support UTF-16 encoding.
///
/// Different fields have different maximum lengths as defined in the RP2350
/// datasheet.  No string can be longer than 127 characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OtpString {
    string: String,
}

impl core::fmt::Display for OtpString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.string)
    }
}

impl TryFrom<&str> for OtpString {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() <= MAX_STRING_LENGTH {
            Ok(OtpString::new(value.to_string()))
        } else {
            Err(Error::StringTooLong(value.len()))
        }
    }
}

impl TryFrom<String> for OtpString {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl OtpString {
    // STRDEF uses 7 bits to represent length
    const MAX_LEN: u8 = MAX_STRING_LENGTH as u8;
    const MAX_ROWS: u8 = 64;

    /// Creates a new OtpString from the given string.
    pub fn new(string: String) -> Self {
        assert!(string.len() <= Self::MAX_LEN as usize, "String is too long");
        Self { string }
    }

    /// Returns true if the string is ASCII only.
    pub fn is_ascii(&self) -> bool {
        self.string.is_ascii()
    }

    /// Returns true if the string is UTF-16
    pub fn is_utf16(&self) -> bool {
        !self.is_ascii()
    }

    /// Returns a reference to the underlying string.
    pub fn string(&self) -> &String {
        &self.string
    }

    pub(crate) fn from_pre_validated_string(string: &str) -> Self {
        Self::try_from(string).expect("internal error - string validation failed")
    }

    /// Returns the string encoded as a `Vec<u16>` suitable for writing to OTP.
    /// ASCII strings are packed two characters per u16, with the first
    /// character in the low byte.  Non-ASCII strings are encoded as UTF-16.
    pub fn to_otp_rows(&self) -> Vec<u16> {
        let otp_row_count = self.otp_row_count();

        if self.string.is_ascii() {
            // Return pairs of bytes as u16, with the first character in the
            // low byte.  This ends up first "on the wire" and first in OTP
            // memory, which is what the bootrom does:
            // dest[i * 2] = (uint8_t)tmp;
            // dest[i * 2 + 1] = (uint8_t)(tmp >> 8);
            let char_count = self.string.len();
            let ascii_vec_len = otp_row_count;
            let mut ascii_vec = Vec::with_capacity(ascii_vec_len as usize);
            let bytes = self.string.as_bytes();
            let mut i = 0;
            while i < char_count {
                let low = bytes[i] as u16;
                let high = if i + 1 < char_count {
                    bytes[i + 1] as u16
                } else {
                    0
                };
                ascii_vec.push(low | (high << 8));
                i += 2;
            }
            ascii_vec
        } else {
            // UTF-16 encoding for non-ascii strings
            self.string.encode_utf16().collect()
        }
    }

    /// Returns the 16-bit STRDEF representation of this string.
    ///
    /// `offset` is the offset from the location of the white label data
    /// structure to the first free row, which is where this string will
    /// be stored.  This is modified to point to the next free row after this
    /// string.
    pub fn to_strdef(&self, offset: &mut u8) -> u16 {
        let is_utf16 = !self.is_ascii();

        let len = self.char_count() as u16;
        let low_byte = (len & 0x7F) | if is_utf16 { 0x80 } else { 0x00 };

        let old_offset = *offset;
        let high_byte = (old_offset as u16) << 8;

        // Update data_offset to point to the next free row after this string
        *offset = old_offset
            .checked_add(self.otp_row_count())
            .expect("internal error - offset overflow");

        low_byte | high_byte
    }

    /// Returns the number of characters in this string, as defined by STRDEF.
    /// We count a surrogate pair as two characters - i.e. u16 rows per UTF-16
    /// char.
    pub(crate) fn char_count(&self) -> u8 {
        let len = if self.is_ascii() {
            self.string.len()
        } else {
            self.string.encode_utf16().count()
        };
        assert!(
            len <= Self::MAX_LEN as usize,
            "String length exceeds maximum of 127 characters"
        );
        len as u8
    }

    /// Returns the number of rows this string will occupy in OTP
    pub fn otp_row_count(&self) -> u8 {
        let row_count = if self.is_ascii() {
            // ASCII strings are packed 2 characters per row
            self.char_count().div_ceil(2)
        } else {
            // UTF-16 strings use 1 row per character
            let utf_count = self.string.encode_utf16().count();
            assert!(
                utf_count <= Self::MAX_LEN as usize,
                "UTF-16 string length exceeds maximum of 127 characters"
            );
            utf_count as u8
        };

        assert!(
            row_count <= Self::MAX_ROWS,
            "Row count exceeds maximum of 64"
        );
        row_count
    }

    /// Returns the offset encoded within a STRDEF row.
    pub(crate) fn offset_from_row(row: u16) -> u8 {
        // MSB encodes the offset
        ((row & 0xFF00) >> 8) as u8
    }

    /// Returns the character count encoded within a STRDEF row.
    pub(crate) fn char_count_from_row(row: u16) -> u8 {
        // LSB bits 0-6 encode the character count
        (row & 0x007F) as u8
    }

    /// Returns true if the STRDEF row indicates a UTF-16 string.
    pub(crate) fn is_ascii_from_row(row: u16) -> bool {
        // LSB bit 7 indicates UTF-16 if set
        (row & 0x0080) == 0
    }

    /// Returns true if the STRDEF row indicates a UTF-16 string.
    pub(crate) fn is_utf16_from_row(row: u16) -> bool {
        !Self::is_ascii_from_row(row)
    }

    /// Returns the total amount of OTP rows occupied by the string defined
    /// in the STRDEF row.
    pub(crate) fn row_count_from_strdef(row: u16) -> u8 {
        let char_count = Self::char_count_from_row(row);
        if Self::is_utf16_from_row(row) {
            char_count
        } else {
            char_count.div_ceil(2)
        }
    }

    /// Extracts an OtpString field from the provided OTP rows at the specified
    /// index.
    ///
    /// Returns `Ok(None)` if the boot flag bit is clear (indicating no string
    /// is present).
    ///
    /// Returns `Err(Error::InternalInconsistency)` if the extracted string's
    /// character count doesn't match the STRDEF.  Passing a non-string field
    /// is a bug in the caller and panics.
    ///
    /// Adds warnings for unusual and suspicious but at least partially
    /// parseable conditions.  Where the string can't be decoded or isn't valid
    /// for the field, it adds a warning and returns `Ok(None)`.
    pub(crate) fn from_otp_data(
        rows: &[u16],
        usb_boot_flags: u16,
        field: &Field,
        utf16_allowed: bool,
        warnings: &mut Vec<String>,
    ) -> Result<Option<Self>, Error> {
        let max_string_len = field.max_length().expect("called with non-string field");
        let field_name = field.name();
        let field_index = field.index();

        let bit_set = (usb_boot_flags & (1 << field_index)) != 0;
        let strdef = rows[field_index];

        if !bit_set {
            // Boot flag bit is clear - field should be None.  This may be
            // valid - if the user has written the OTP data for this row, but
            // not yet set the boot flag bit, so the bootloader doesn't use it.
            if strdef != 0 {
                warnings.push(format!(
                    "{}: boot flag bit {} clear but row contains non-zero STRDEF 0x{:04X}",
                    field_name, field_index, strdef
                ));
            }
            return Ok(None);
        }

        // Boot flag bit is set - we should have a valid string
        if strdef == 0 {
            warnings.push(format!(
                "{}: boot flag bit {} set but STRDEF is zero",
                field_name, field_index
            ));
            return Ok(None);
        }

        // Extract fields from STRDEF
        let offset = Self::offset_from_row(strdef) as usize;
        let char_count = Self::char_count_from_row(strdef) as usize;
        let is_utf16 = Self::is_utf16_from_row(strdef);

        // Validate offset is within bounds and after struct fields
        if offset < NUM_FIELDS {
            warnings.push(format!(
                "{}: string offset {} is less than {} (must be after struct fields)",
                field_name, offset, NUM_FIELDS
            ));
            return Ok(None);
        }

        // Validate UTF-16 usage if not allowed
        if is_utf16 && !utf16_allowed {
            warnings.push(format!(
                "{}: UTF-16 string not allowed but STRDEF indicates UTF-16 encoding",
                field_name
            ));
            // We will continue to parse in this instance.
        }

        // Calculate how many rows the string occupies
        let row_count = if is_utf16 {
            // UTF-16 strings use 1 row per character
            char_count
        } else {
            // ASCII strings are packed 2 characters per row
            char_count.div_ceil(2)
        };

        // Validate we have enough rows
        let end_offset = offset + row_count;
        if end_offset > rows.len() {
            warnings.push(format!(
                "{}: string at offset {} with {} rows exceeds available {} rows",
                field_name,
                offset,
                row_count,
                rows.len()
            ));
            return Ok(None);
        }

        // Extract the string data rows
        let string_rows = &rows[offset..end_offset];

        // Decode the string based on encoding
        let string = if is_utf16 {
            // UTF-16 decoding - one character per u16
            match String::from_utf16(string_rows) {
                Ok(s) => s,
                Err(e) => {
                    warnings.push(format!(
                        "{}: invalid UTF-16 data at offset {}: {}",
                        field_name, offset, e
                    ));
                    return Ok(None);
                }
            }
        } else {
            // ASCII decoding - two characters per u16, first character in low byte
            let mut bytes = Vec::new();
            for &row in string_rows {
                let low = (row & 0xFF) as u8;
                let high = ((row >> 8) & 0xFF) as u8;
                bytes.push(low);
                bytes.push(high);
            }

            // Check for non-zero padding beyond the declared character count
            if bytes.len() > char_count {
                let padding = &bytes[char_count..];
                if padding.iter().any(|&b| b != 0) {
                    warnings.push(format!(
                        "{}: non-zero padding bytes found after declared length {}",
                        field_name, char_count
                    ));
                }
            }

            // Trim to exact character count (removes padding zeros)
            bytes.truncate(char_count);

            match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => {
                    warnings.push(format!(
                        "{}: invalid ASCII/UTF-8 data at offset {}: {}",
                        field_name, offset, e
                    ));
                    return Ok(None);
                }
            }
        };

        // An ASCII STRDEF holds one byte per character so a byte above 0x7F
        // is invalid even where the bytes form valid UTF-8.
        if !is_utf16 && !string.is_ascii() {
            warnings.push(format!(
                "{}: non-ASCII data in ASCII string at offset {}",
                field_name, offset
            ));
            return Ok(None);
        }

        // Drop a UTF-16 string in an ASCII-only field unless its characters
        // are all ASCII.
        if !utf16_allowed && !string.is_ascii() {
            warnings.push(format!(
                "{}: non-ASCII characters in ASCII-only field",
                field_name
            ));
            return Ok(None);
        }

        // Check length.  Drop a string longer than the JSON schema allows.
        if string.chars().count() > max_string_len {
            warnings.push(format!(
                "{}: extracted string length {} exceeds maximum of {}",
                field_name,
                string.chars().count(),
                max_string_len
            ));
            return Ok(None);
        }

        let otp_string = OtpString::try_from(string)?;

        // Sanity check the extracted string against the STRDEF.  A UTF-16
        // string of only ASCII characters becomes an ASCII OtpString so the
        // encodings can differ but the character counts can't.
        if otp_string.char_count() as usize != char_count {
            return Err(Error::InternalInconsistency(format!(
                "{}: extracted string character count does not match STRDEF",
                field_name
            )));
        }

        Ok(Some(otp_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_string_to_otp_string() {
        let s = OtpString::try_from("hello").unwrap();
        assert_eq!(s.char_count(), 5);
        assert!(s.is_ascii());
        assert!(!s.is_utf16());
        assert_eq!(s.otp_row_count(), 3);
        let otp_rows = s.to_otp_rows();
        assert_eq!(otp_rows, vec![0x6568, 0x6c6c, 0x006f]); // 'H' 'e', 'l' 'l', 'o' 0
    }

    #[test]
    fn test_string_too_long() {
        let long_string = "a".repeat(128);
        let res = OtpString::try_from(long_string);
        assert!(res.is_err(), "Expected error for string too long");
    }

    #[test]
    fn test_utf16_string_to_otp_string() {
        let s = OtpString::try_from("héllo").unwrap(); // 'é' is non
        assert_eq!(s.char_count(), 5);
        assert!(!s.is_ascii());
        assert!(s.is_utf16());
        assert_eq!(s.otp_row_count(), 5);
        let otp_rows = s.to_otp_rows();
        assert_eq!(otp_rows, vec![0x0068, 0x00e9, 0x006c, 0x006c, 0x006f]);

        let strdef = s.to_strdef(&mut 10);
        assert_eq!(strdef, 0x0A85); // offset 10, length 5, UTF-16 flag set
    }

    #[test]
    fn test_to_otp_rows_ascii() {
        let s = OtpString::try_from("ABCD").unwrap();
        let rows = s.to_otp_rows();
        assert_eq!(rows, vec![0x4241, 0x4443]); // 'A' 'B', 'C' 'D'
        let strdef = s.to_strdef(&mut 5);
        assert_eq!(strdef, 0x0504); // offset 5, length
        let otp_row_count = s.otp_row_count();
        assert_eq!(otp_row_count, 2);

        let s = OtpString::try_from("abcde").unwrap();
        let rows = s.to_otp_rows();
        assert_eq!(rows, vec![0x6261, 0x6463, 0x0065]); // 'a' 'b', 'c' 'd', 'e' 0
        let strdef = s.to_strdef(&mut 3);
        assert_eq!(strdef, 0x0305); // offset 3, length
        let otp_row_count = s.otp_row_count();
        assert_eq!(otp_row_count, 3);
    }

    #[test]
    fn test_to_otp_rows_utf16() {
        let s = OtpString::try_from("héllo").unwrap();
        let rows = s.to_otp_rows();
        assert_eq!(rows, vec![0x0068, 0x00e9, 0x006c, 0x006c, 0x006f]);
        let strdef = s.to_strdef(&mut 8);
        assert_eq!(strdef, 0x0805 | 0x0080);
        let char_count = s.char_count();
        assert_eq!(char_count, 5);
        let otp_row_count = s.otp_row_count();
        assert_eq!(otp_row_count, 5);

        // Surrogate pair example: 😀 (U+1F600) is encoded as D83D DE00 in UTF-16
        let s = OtpString::try_from("hello😀").unwrap();
        let rows = s.to_otp_rows();
        assert_eq!(
            rows,
            vec![0x0068, 0x0065, 0x006c, 0x006c, 0x006f, 0xd83d, 0xde00]
        );
        let strdef = s.to_strdef(&mut 12);
        assert_eq!(strdef, 0x0C07 | 0x0080); // 7 rows (including surrogate pair)
        let char_count = s.char_count();
        assert_eq!(char_count, 7); // 7 characters as surrogate pair counts double
        let otp_row_count = s.otp_row_count();
        assert_eq!(otp_row_count, 7);
    }
}
