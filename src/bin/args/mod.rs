// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use clap::{CommandFactory, Parser};

/// Arguments for the pico-otp command line tool
#[derive(Parser, Debug, Default)]
#[command(group = clap::ArgGroup::new("input")
    .required(true)
    .args(["json_file", "otp_dump_file"]))]
pub struct Args {
    /// Path to the JSON whitelabel config file
    #[clap(
        short = 'j',
        alias = "json",
    )]
    pub json_file: Option<String>,

    /// Output the OTP rows to this file
    #[clap(
        short = 'o',
        alias = "output",
        requires = "json_file",
    )]
    pub otp_output_file: Option<String>,

    /// Path to the OTP dump binary file.  Should contain only the ECC OTP rows
    /// that include the whitelabel data (that pointed to by USB_WHITE_LABEL_ADDR,
    /// 0x05c).
    #[clap(
        short = 'd',
        alias = "dump",
        requires = "boot_flags",
    )]
    pub otp_dump_file: Option<String>,

    /// Output the generated JSON to this file
    #[clap(
        short = 'e',
        alias = "json-output",
        requires = "otp_dump_file",
    )]
    pub json_output_file: Option<String>,

    /// USB boot flags associated with the OTP dump.  Must be the non-ECC (raw)
    /// value from USB_BOOT_FLAGS (0x059), USB_BOOT_FLAGS_R1 (0x05A) or
    /// USB_BOOT_FLAGS_R2 (0x05B).  THe caller should ensure these different
    /// values are consistent.
    #[clap(
        short = 'b',
        alias = "boot-flags",
        requires = "otp_dump_file",
        value_parser = Args::parse_hex,
        value_name = "0xHEXVAL",
    )]
    pub boot_flags: Option<u32>,
}

impl Args {
    pub(crate) fn print_help() {
        Args::command().print_help().unwrap();
    }
        
    fn parse_hex(s: &str) -> Result<u32, String> {
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u32::from_str_radix(hex, 16)
                .map_err(|e| format!("Invalid hex value: {}", e))
        } else {
            Err("Value must start with 0x or 0X".to_string())
        }
    }
}
