# pico-otp

A Rust crate and tool for generating and parsing OTP data for the Raspberry Pi RP2350/Pico 2 microcontroller.  It specifically targets generating and decoding USB white labelling and related information.

## Warning

⚠️ WRITING TO OTP MEMORY IS PERMANENT AND MAY IRREPARABLY BRICK YOUR DEVICE IF DONE INCORRECTLY. PROCEED WITH CAUTION ⚠️

There is no warranty provided with this software.  Use at your own risk.  See the [LICENSE file](LICENSE.md) for details.

## Why?

Raspberry Pi's [picotool](https://github.com/raspberrypi/picotool) can be used to read and write OTP white label data.

`pico-otp's` primary purpose is to provide a programmatic API and library for working with this data, including generating the required OTP row data from JSON files, and converting OTP row data from real devices back into JSON format.

`pico-otp` is used by [`pico⚡flash`](https://picoflash.org), which provides an accessbile web interface for white labelling and also erasing, reading and writing RP2350-based devices.

In addition, it useful to have a well commented and thoroughly tested parallel implemenation of the white labelling logic, to help understand and verify the behaviour of picotool and the RP2350, as some of Raspberry Pi's documentation on this topic was found to be incomplete or unclear.

## Example USB White Label Config

```json
{
    "$schema": "https://raw.githubusercontent.com/raspberrypi/picotool/develop/json/schemas/whitelabel-schema.json",
    "device": {
        "manufacturer": "piers.rocks",
        "product": "pico-otp",
        "serial_number": "1234abcd"
    },
    "scsi": {
        "vendor": "piersrks",
        "product": "pico-otp",
        "version": "v123"
    },
    "volume": {
        "label": "PIERS.ROCKS",
        "redirect_url": "https://piers.rocks/",
        "redirect_name": "piers.rocks",
        "model": "pico-otp",
        "board_id": "pico-otp board id"
    }
}
```

## Command Line Tool - Example Usage 

```sh
cargo run --bin pico-otp -- -i json/sample-wl.json -o /tmp/otp.bin
```

Sample output:

```
Processed json/sample-wl.json and wrote OTP rows to /tmp/otp.bin as LE data
-----
USB boot flags: 0x0040FF77
-----
To use this output to white label your RP2350:
  - Write the contents of the output to file to OTP memory as ECC rows
    starting at a known free OTP region, typically 0x100
  - Write the offset you selected to OTP row 0x05c (USB_WHITE_LABEL_ADDR)
  - Write the USB boot flags 0x0040FF77 to OTP rows 0x059, 0x5a and 0x5b
    (USB_BOOT_FLAGS, USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2)
    as raw (not ECC) data
```

## Rust Crate - Example Usage 

```rust
use pico_otp::WhiteLabelStruct;

// Load the JSON file
let json_str = std::fs::read_to_string("json/sample-wl.json").unwrap();

// Parse it and create the struct
let whitelabel = WhiteLabelStruct::from_json(&json_str).unwrap();

// Check some fields
println!("USB VID: {:#04x}", whitelabel.vendor_id.unwrap());
println!("USB PID: {:#04x}", whitelabel.product_id.unwrap());

// Generate the required OTP row data
let otp_data = whitelabel.to_otp_rows();

// And the boot flags
let usb_boot_flags = whitelabel.usb_boot_flags();
```

## Features

- Supports all 16 USB white labelling fields
- Supports the same JSON schema as Raspberry Pi's [picotool](https://github.com/raspberrypi/picotool)
- Encodes white label data into OTP ECC rows as human readable or binary data
- Decodes OTP data read from existing white labelled device back into JSON format
- Handles ASCII and UTF-16 USB strings encoding, including UTF-16 surrogates (like 😀)
- `no-std` support, for use in WASM and embedded environments
- Command line tool provided for generating OTP data from JSON files
- Comprehensive unit tests to ensure correctness of data encoding and decoding

## Technical Details

See the [Technical Overview](TECHNICAL.md) file for more information on the technical details of RP2350 One Time Programmable memory.

See the [USB White Labelling](docs/USB.md) document for more information on the RP2350's USB white labelling specifically.

## License

This project is licensed under the MIT OR Apache 2.0 Licenses, at your option. See the [LICENSE file](LICENSE.md) for details.

## Acknowledgements

Raspberry Pi's OTP white labelling JSON schema has been reproduced [here](json/whitelabel-schema.json) and is licensed at described [here](json/LICENSE-SCHEMA.md)
