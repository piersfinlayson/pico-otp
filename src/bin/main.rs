// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use clap::Parser;
use pico_otp::OtpData;

mod args;
use args::Args;

fn main() {
    let args = Args::parse();
    std::process::exit(run(&args));
}

fn usage() {
    Args::print_help();
}

fn run(args: &Args) -> i32 {
    if args.json_file.is_some() {
        return process_json(&args);
    } else if args.otp_dump_file.is_some() {
        return process_otp_dump(&args);
    } else {
        usage();
        return 1;
    }
}

fn process_otp_dump(args: &Args) -> i32 {
    // Get the args
    let otp_dump_file = args.otp_dump_file.as_ref().unwrap();
    let output_file = match &args.json_output_file {
        Some(name) => Some(name),
        None => None,
    };
    let usb_boot_flags = match args.boot_flags {
        Some(flags) => flags,
        None => {
            eprintln!("USB boot flags must be specified when processing an OTP dump");
            return 1;
        }
    };

    // Read the OTP dump file
    let otp_dump = match std::fs::read(otp_dump_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read OTP dump file: {e}");
            return 1;
        }
    };
    if !otp_dump.len().is_multiple_of(2) {
        eprintln!("OTP dump file has an invalid length (must be even number of bytes)");
        return 1;
    }

    // Turn it into u16 rows, from little endian format
    let otp_dump: Vec<u16> = otp_dump
        .chunks(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Process it
    let otp_data = match OtpData::from_white_label_data(usb_boot_flags, &otp_dump, true) {
        Ok(od) => {
            println!("Parsed OTP dump successfully");
            od
        },
        Err(e) => {
            eprintln!("Failed to parse OTP dump: {e}");
            return 1;
        }
    };

    // Serialize to JSON
    let json = match otp_data.to_json() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize to JSON: {e}");
            return 1;
        }
    };

    // Pretty print
    let json_str = match serde_json::to_string_pretty(&json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize JSON to string: {e}");
            return 1;
        }
    };

    if let Some(output_path) = output_file {
        // Write to file
        match std::fs::write(output_path, json_str) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to write JSON output file: {e}");
                return 1;
            }
        }
        println!("Wrote JSON output to {}", output_path);
    } else {
        // Print JSON to stdout
        println!("{}", json_str);
    }

    0
}

fn process_json(args: &Args) -> i32 {
    // Get the args
    let json_file = args.json_file.as_ref().unwrap();
    let output_file = match &args.otp_output_file {
        Some(name) => Some(name),
        None => None,
    };

    // Read the JSON file
    let json_content = std::fs::read_to_string(json_file).expect("Failed to read JSON file");

    // Single step to turn it into OtpData
    let otp_data = match OtpData::from_json(&json_content) {
        Ok(od) => {
            if output_file.is_none() {
                println!("Parsed white label data from {json_file} successfully");
            }
            od
        }
        Err(e) => {
            eprintln!("Failed to parse JSON: {e}");
            return 1;
        }
    };

    // Get the OTP rows and boot flags
    let boot_flags = otp_data.usb_boot_flags();

    if let Some(output_path) = output_file {
        // Get the bytes as a flat array of u8s, 2 for each ECC row
        let bytes = otp_data.to_le_ecc_bytes();

        // Write to file
        match std::fs::write(output_path, &bytes) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to write to output file: {e}");
                return 1;
            }
        }
        println!("Processed {json_file} and wrote OTP rows to {output_path} as LE data");
        println!("-----");
        println!("USB boot flags: {boot_flags:#010X}");
        println!("-----");
        println!("To use this output to white label your RP2350:");
        println!("  - Write the contents of the output to file to OTP memory as ECC rows");
        println!("    starting at a known free OTP region, typically 0x100");
        println!("  - Write the offset you selected to OTP row 0x05c (USB_WHITE_LABEL_ADDR)");
        println!(
            "  - Write the USB boot flags {boot_flags:#010X} to OTP rows 0x059, 0x5a and 0x5b"
        );
        println!("    (USB_BOOT_FLAGS, USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2)");
        println!("    as raw (not ECC) data");
        println!("-----");
        println!(
            "PROCEED WITH CAUTION - WRITING TO OTP IS PERMANENT AND MAY IRREPARABLY BRICK YOUR DEVICE"
        );
    } else {
        let otp_rows = otp_data.rows();
        println!("USB Boot Flags: {boot_flags:#010X}");
        println!("Total OTP row count: {}", otp_rows.len());

        // Print OTP rows in hex as u16s
        print!("OTP Rows - write at some offset, such as 0x100:\n");
        for (ii, row) in otp_rows.iter().enumerate() {
            let row_index = ii + 0x100;
            println!("  {row_index:#05X}: {row:#06X}");
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let test_json = r#"{
            "device": {
                "vid": "0x4660",
                "pid": "0x2136",
                "manufacturer": "Test Manufacturer",
                "product": "Test Product",
                "serial_number": "1234567890"
            }
        }"#;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let json_path = temp_dir.path().join("test-wl.json");
        std::fs::write(&json_path, test_json).expect("Failed to write test JSON");

        // Set args
        let args = Args {
            json_file: Some(json_path.to_str().unwrap().to_string()),
            ..Default::default()
        };
        let rc = run(&args);
        assert_eq!(rc, 0);
    }
}
