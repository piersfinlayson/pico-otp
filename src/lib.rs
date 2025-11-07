// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! pico-otp
//!
//! A library for generating and decoding OTP (One-Time Programmable) binary
//! data for the Raspberry Pi Pico 2 and other RP2350-based devices.
//!
//! This library provides functionality to create OTP binary blobs that can be
//! used to configure device-specific settings such as USB identifiers, SCSI
//! information, and volume labels. It supports whitelabel configurations
//! defined in JSON format, allowing for easy customization of device
//! parameters.
//! 
//! It can also be used to decode OTP binary data back into native Rust objects
//! and Raspberry Pi's standard USB white labelling JSON schema.
//!
//! It is `no_std` compatible, making it suitable for WASM, embedded and other
//! minimal environments.
//! 
//! It is used by [`pico⚡flash`](https://picoflash.org) to apply whitelabelling
//! configurations to Raspberry Pi Pico 2 and other RP2350 devices.
//!
//! # Features
//!
//! - Generate OTP binary data from whitelabel JSON configurations.
//! - Parses and validates OTP binary data dumps, extracting whitelabel
//!   information.
//! - Supports `picotool` whitelabel schema.
//! - `no_std` compatible (requires `alloc`).
//!
//! # Example
//!
//! Generate the OTP binary data from a whitelabel JSON file:
//!
//! ```rust
//! use pico_otp::WhiteLabelStruct;
//!
//! // Load the JSON file
//! let json_str = std::fs::read_to_string("json/sample-wl.json").unwrap();
//!
//! // Parse it and create the struct
//! let whitelabel = WhiteLabelStruct::from_json(&json_str).unwrap();
//!
//! // Check some fields
//! println!("USB VID: {:#04x}", whitelabel.vendor_id.unwrap());
//! println!("USB PID: {:#04x}", whitelabel.product_id.unwrap());
//!
//! // Generate the required OTP row data
//! let otp_data = whitelabel.to_otp_rows();
//!
//! // And the boot flags
//! let usb_boot_flags = whitelabel.usb_boot_flags();
//!
//! // Now, write the OTP data at a suitable row index, using ECC mode.
//! // 0x100 is a common offset choice, as it's what picotool uses.
//! //
//! // You will also need to:
//! // - Set the USB_WHITE_LABEL_ADDR row (0x05c) to point to 0x100, or
//! //   whatever row you chose.
//! // - Write usb_boot_flags to rows 0x059, 0x05a and 0x05b - USB_BOOT_FLAGS,
//! //   USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2 respectively to enable this
//! //   whitelabel data.  You must write this as non-ECC (raw) data.
//! ```

#![no_std]

extern crate alloc;

pub mod whitelabel;
pub use whitelabel::{OtpString, WhiteLabelStruct};
