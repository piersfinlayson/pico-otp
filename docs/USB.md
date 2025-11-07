# USB White Labelling

If you aren't familiar with the RP2350's OTP support, read the [Technical Overview](TECHNICAL.md) or [RP2350 datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) first.

## Overview

The RP2350's OTP memory can be used to store USB white label information, allowing for customization of the device's identity when operating in BOOTSEL mode.  To be clear, this only impacts BOOTSEL mode - if you want to implement a USB white label device with custom firmware, you are able to do this without using OTP memory.  However, to customize the USB identity of the BOOTSEL mode itself - for example to allow your customers to use BOOTSEL to interact with your RP2350 device - you may wish to white label it.

Strictly, more than just USB information can be white labelled - the volume and files presented by the BOOTSEL's mass storage device can also be customized.

The complete list of data that can be white labelled is:

| Index | Data | Default | Type of data | Max Chars |
|-------|------|---------|--------------|------------|
| 0 | USB device vendor ID (VID) | 0x2e8a | U16 | - |
| 1 | USB device product ID (PID) | 0x000f | U16 | - |
| 2 | USB device version | 0x0100 | BCD16 | - |
| 3 | USB device language ID | 0x0409 | U16 | - |
| 4 | USB device manufacturer | Raspberry Pi | _STRDEF_U | 30 |
| 5 | USB device product | RP2350 Boot | _STRDEF_U | 30 |
| 6 | USB device serial number | DEVICE_ID | _STRDEF_U | 30 |
| 7 | USB device attributes & max power | 0xfa80 | U16_M | - |
| 8 | Mass storage volume label | RP2350 | _STRDEF_A | 11 |
| 9 | SCSI inquiry vendor | RPI | _STRDEF | 8 |
| 10 | SCSI inquiry product | RP2350 | _STRDEF_A | 16 |
| 11 | SCSI inquiry version | 1 | _STRDEF_A | 4 |
| 12 | index.htm redirect URL | https://some.pi.url | _STRDEF_A | 127 |
| 13 | index.htm redirect name | raspberrypi.com | _STRDEF_A | 127 |
| 14 | uf2.txt model | Raspberry Pi RP2350 | _STRDEF_A | 127 |
| 15 | uf2.txt board ID | RP2350 | _STRDEF_A | 127 |

## Storing Data

There are a number of steps involved in storing USB white label data in the RP2350's OTP memory:

1. Fill in the row index of the start of the white label structure (see step 2) in row USB_WHITE_LABEL_ADDR (0x05c).

2. Create the USB white label structure, using 16 contiguous OTP ECC rows, filling in each of the values for the specific entries above you wish to define.  See [Types](#types) for how to do this.

3. Mark the values of the specific entries above you wish to use as being valid in USB_BOOT_FLAGS, USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2 (0x059, 0x05a and 0x05b).  These are raw (non-ECC) rows, so each flag can be set individually at different times.

4. Mark that the white label address is valid by setting the appropriate flag in USB_BOOT_FLAGS, USB_BOOT_FLAGS_R1 and USB_BOOT_FLAGS_R2.

There are different orders in which these steps can be performed.  The bootloader will only attempt to use the white label data if the appropriate white label address and specific valid flags are set.  If it comes across invalid data, it will simply ignore it and use the default values.

However, to avoid hitting possible bugs in the bootloader, and therefore possible permanent bricking of the RP2350, it is advisable to deviate as litle as possible from the proscribed order.

## Types

The data types used in the table above are defined as follows:

### U16

An unsigned 16-bit integer.

### U16_M

Contains the combined USB device attributes and maximum power values with:
- Low byte: bmAttributes
- High byte: maxPower

These correspond to the USB standard definitions for these fields.

### BCD16

A two-byte Binary-Coded Decimal (BCD) value.  For example, version 2.34 would be represented as 0x0234 ([0x34, 0x02] within the row).

### _STRDEF

A two byte value:
- Low byte:
  - Bits [6:0] indicate the length of the string, in characters.
  - Bit 7 indicates whether the string characters are encoded as 2 byte (UTF-16) or 1 byte (ASCII).  A 1 indicates UTF-16, a 0 indicates ASCII.
- High byte: Indicates the location of the beginning of the string data, as an offset from the location of the _USB white label structure_ (i.e. the structure the _STRDEF is itself an offset from ).

Where _STRDEF is used in the table above, it indicates only ASCII is supported.

With ASCII encoding, each character is stored as a single byte.  Hence, each row supports up to ASCII 2 characters.  Any unused byte within a row should be set to 0.

### _STRDEF_U

_STRDEF, optionally supported Unicode or ASCII strings.  This is limited to:
- USB device manufacturer
- USB device product
- USB device serial number

These types of strings can be encoded as either ASCII or UTF-16, depending on the value of bit 7 of the low byte.  ASCII characters are packed 2 to a row, while UTF-16 characters use the full row per character.

### Maximum String Lengths

As can be seen above, each string field has a maximum length.  However, there is also a maximum length of _all_ string data combined, as the offsets used in the _STRDEF values are only a single byte.

Therefore, all strings must _start_ within 255 rows of the start of the USB white label structure.  Hence the total length of all strings except the longest, is 239 rows, accounting for the fact that the white label structure is 16 rows.

If ASCII is exclusively used, this gives a maximum capacity of roughly 478 characters (give or take), plus the last string, with Unicode support reducing this. 

## JSON Format

Raspberry Pi's [picotool](https://github.com/raspberrypi/picotool) is a command-line tool for working with Pico-based devices.  It includes support for reading and writing OTP white label data.  It uses a JSON format to define the white label data:

```json
{
    "$schema": "https://raw.githubusercontent.com/raspberrypi/picotool/develop/json/schemas/whitelabel-schema.json",
    "device": {
        "vid": "0x2e8b",
        "pid": "0x000e",
        "bcd": 2.15,
        "lang_id": "0x0c09",
        "manufacturer": "zß水🍌 Test's Pis",
        "product": "Test RP2350?",
        "serial_number": "notnecessarilyanumber",
        "max_power": "0x20",
        "attributes": "0xe0"
    },
    "scsi": {
        "vendor": "TestPi",
        "product": "MyPi",
        "version": "v897"
    },
    "volume": {
        "label": "TestPi Boot",
        "redirect_url": "https://www.raspberrypi.com/news/",
        "redirect_name": "Some News About Stuff",
        "model": "My Test Pi",
        "board_id": "TPI-RP2350"
    }
}
```

This tool uses the same format and schema.