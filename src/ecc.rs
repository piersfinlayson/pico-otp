// Copyright (C) 2026 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Contains the OTP row ECC encoder.

// The parity masks from `s_otp_calculate_ecc()` in the RP2350 datasheet.
// Bit 16 + i of a row holds the even parity of the row bits mask i selects.
// The last mask selects the other five parity bits as well as the data.
const PARITY_MASKS: [u32; 6] = [
    0b0000001010110101011011,
    0b0000000011011001101101,
    0b0000001100011110001110,
    0b0000000000011111110000,
    0b0000001111100000000000,
    0b0111111111111111111111,
];

/// Returns the 24 bits an OTP row holds when `value` is written to it with
/// ECC.
///
/// - Bits 15:0 hold `value`.
/// - Bits 21:16 hold six parity bits, calculated as `s_otp_calculate_ecc()`
///   does in section 13.6.2 of the RP2350 datasheet.
/// - Bits 23:22 are clear.
///
/// A row that had a bit set before programming can instead hold the returned
/// value with all 24 bits inverted, so bits 23:22 are set. This is bit repair
/// by polarity, in section 13.6.1 of the datasheet.
pub const fn ecc_encode(value: u16) -> u32 {
    let mut row = value as u32;
    let mut i = 0;
    while i < PARITY_MASKS.len() {
        row |= ((row & PARITY_MASKS[i]).count_ones() & 1) << (16 + i);
        i += 1;
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    // picotool's otp_calculate_ecc() from main.cpp. It masks the data once for
    // each Hamming bit, then works out the overall parity bit from the data
    // and those five bits.
    fn picotool_ecc(value: u16) -> u32 {
        let x = value as u32;
        let parity = |v: u32| v.count_ones() & 1;
        let p0 = parity(x & 0b1010110101011011);
        let p1 = parity(x & 0b0011011001101101);
        let p2 = parity(x & 0b1100011110001110);
        let p3 = parity(x & 0b0000011111110000);
        let p4 = parity(x & 0b1111100000000000);
        let p5 = parity(x) ^ p0 ^ p1 ^ p2 ^ p3 ^ p4;
        let p = p0 | (p1 << 1) | (p2 << 2) | (p3 << 3) | (p4 << 4) | (p5 << 5);
        x | (p << 16)
    }

    #[test]
    fn test_ecc_encode_matches_picotool() {
        for value in 0..=u16::MAX {
            assert_eq!(ecc_encode(value), picotool_ecc(value), "value {value:#06x}");
        }
    }

    // Rows returned by both of these C functions, compiled with Apple clang 21:
    // - `s_otp_calculate_ecc()` from section 13.6.2 of the RP2350 datasheet,
    //   build-version d126e9e.
    // - `otp_calculate_ecc()` from picotool's main.cpp at commit b02e66c.
    // The table has every single-bit value. Each parity bit is an XOR of data
    // bits, so the single-bit rows determine every other row.
    const C_ROWS: [(u16, u32); 21] = [
        (0x0000, 0x000000),
        (0x0001, 0x230001),
        (0x0002, 0x250002),
        (0x0004, 0x260004),
        (0x0008, 0x070008),
        (0x0010, 0x290010),
        (0x0020, 0x2a0020),
        (0x0040, 0x0b0040),
        (0x0080, 0x2c0080),
        (0x0100, 0x0d0100),
        (0x0200, 0x0e0200),
        (0x0400, 0x2f0400),
        (0x0800, 0x310800),
        (0x1000, 0x321000),
        (0x2000, 0x132000),
        (0x4000, 0x344000),
        (0x8000, 0x158000),
        (0xffff, 0x1effff),
        (0x1234, 0x191234),
        (0x5678, 0x285678),
        (0xabcd, 0x11abcd),
    ];

    #[test]
    fn test_ecc_encode_matches_c() {
        for (value, row) in C_ROWS {
            assert_eq!(ecc_encode(value), row, "value {value:#06x}");
        }
    }

    // Rows 0x000 to 0x00b of a stock RP2350 A4, read with ECC and then raw, as
    // listed in `docs/TECHNICAL.md`.
    const CHIP_ROWS: [(u16, u32); 12] = [
        (0x5b6b, 0x145b6b),
        (0x2f65, 0x2a2f65),
        (0x9c23, 0x159c23),
        (0xde3f, 0x27de3f),
        (0x6986, 0x346986),
        (0xfd39, 0x34fd39),
        (0x45eb, 0x1a45eb),
        (0xf33c, 0x21f33c),
        (0xb1e3, 0x32b1e3),
        (0xecfb, 0x09ecfb),
        (0xd5cc, 0x37d5cc),
        (0x372e, 0x23372e),
    ];

    #[test]
    fn test_ecc_encode_matches_chip() {
        for (value, row) in CHIP_ROWS {
            assert_eq!(ecc_encode(value), row, "value {value:#06x}");
        }
    }
}
