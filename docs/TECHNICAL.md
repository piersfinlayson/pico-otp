# Technical Overview of RP2350 OTP

THe RP2350's OTP functionionality is documented in the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf).

## Overview

There are 4096 rows of OTP memory.

Each row holds 24 bits of data.

Each bit in OTP can only be written from a 0 to a 1, and only once.

You either write to a row in ECC mode, or raw mode.

ECC is used for most data, and reserves the top 8 bits for ECC parity (strictly 6 bits for Hamming ECC protection and 2 bits of bit polarity reversal protection - see the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) for the gory details).

In ECC mode you only get access to the bottom 16 bits of each row, with the top 8 bits used for ECC.

In raw mode, the full 24 bits are available, but there is no ECC protection.

You can read back an ECC row in raw mode, to view the full 24 bits, but to achieve automatic writes of ECC data (i.e. the ECC parity bits are calculated and written automatically), you must write in ECC mode.

Raw mode is primarily used for storing data that does not require ECC protection, for example when it is expected that bits will be written at different times to each other - meaning ECC data would become invalid for that row.

The RP2350 uses raw mode for storing data such as flags enabling/disabling features, and indicating whether other data in OTP is valid or not.  It combines raw mode with other mechanisms to provide protection for this data - such as 3 copies with majority voting.  (In fact 8 copies with 3 of 8 voting is used for the most critical flags.)

ECC and raw mode data MUST NOT be mixed within a single row, and in fact, you are STRONGLY RECOMMENDED to avoid mixing it in adjacent even/odd row pairs, due to errata E17.

The RP2350 uses OTP for various purposes, including the device's unique serial number, factory oscillator calibration data, etc.

The RP2350 also supports other OTP data, for example, to white-label the bootloader's USB identification information.

OTP data is arranged into 64 pages of 64 rows each matching the total 4096 rows.

## Notation

In this document, and throughout this library, we refer to row index, not address, for OTP data.

Hence, 0x05c refers to the USB_WHITE_LABEL_ADDR row index.  This is an ECC row that points to a contiguous set of 16 rows, that themselves contain either USB white label data, or a "pointer" to USB white label data elsewhere in OTP, depending on the index into that set of 16. 

Row indices can be converted to memory addresses, using the memory mapped regions described in [Basic Access](#basic-access).

## Visualization

Here are the first 16 rows (0x000 to 0x00F) of OTP memory, from a stock RP2350, A4 stepping, read in ECC mode:

```
Row   Data    Binary (MSB→LSB)    ASCII
000  0x5b6b  0101 1011 0110 1011   k[  
001  0x2f65  0010 1111 0110 0101   e/  
002  0x9c23  1001 1100 0010 0011   #.  
003  0xde3f  1101 1110 0011 1111   ?.  
004  0x6986  0110 1001 1000 0110   .i  
005  0xfd39  1111 1101 0011 1001   9.  
006  0x45eb  0100 0101 1110 1011   .E  
007  0xf33c  1111 0011 0011 1100   <.  
008  0xb1e3  1011 0001 1110 0011   ..  
009  0xecfb  1110 1100 1111 1011   ..  
00a  0xd5cc  1101 0101 1100 1100   ..  
00b  0x372e  0011 0111 0010 1110   .7  
00c  0x0000  0000 0000 0000 0000   ..  
00d  0x0000  0000 0000 0000 0000   ..  
00e  0x0000  0000 0000 0000 0000   ..  
00f  0x0000  0000 0000 0000 0000   ..  
```

The first 4 rows contain the RP2350's serial number (not guaranteed to be unique, but highly likely to be so).  This device reports it serial number via USB as `DE3F9C232F655B6B`.

Rows 0x004 through 0x00b the private, per-device, random number.

Note that data is stored in little-endian format, as normal, hence the values above have had their bytes reversed for display - 0x5b6b is actually stored in the row as [0x6b, 0x5b].

And, the same data in raw mode:

```
Row     Data                Binary (MSB→LSB)              ASCII
000  0x00145b6b  0000 0000 0001 0100 0101 1011 0110 1011  k[.. 
001  0x002a2f65  0000 0000 0010 1010 0010 1111 0110 0101  e/*. 
002  0x00159c23  0000 0000 0001 0101 1001 1100 0010 0011  #... 
003  0x0027de3f  0000 0000 0010 0111 1101 1110 0011 1111  ?.'. 
004  0x00346986  0000 0000 0011 0100 0110 1001 1000 0110  .i4. 
005  0x0034fd39  0000 0000 0011 0100 1111 1101 0011 1001  9.4. 
006  0x001a45eb  0000 0000 0001 1010 0100 0101 1110 1011  .E.. 
007  0x0021f33c  0000 0000 0010 0001 1111 0011 0011 1100  <.!. 
008  0x0032b1e3  0000 0000 0011 0010 1011 0001 1110 0011  ..2. 
009  0x0009ecfb  0000 0000 0000 1001 1110 1100 1111 1011  .... 
00a  0x0037d5cc  0000 0000 0011 0111 1101 0101 1100 1100  ..7. 
00b  0x0023372e  0000 0000 0010 0011 0011 0111 0010 1110  .7#. 
00c  0x00000000  0000 0000 0000 0000 0000 0000 0000 0000  .... 
00d  0x00000000  0000 0000 0000 0000 0000 0000 0000 0000  .... 
00e  0x00000000  0000 0000 0000 0000 0000 0000 0000 0000  .... 
00f  0x00000000  0000 0000 0000 0000 0000 0000 0000 0000  .... 
```

Here we can clearly see the ECC parity bits in the top 8 bits of each row in ECC mode.

To view the OTP data on your RP2350, head over to [pico⚡flash](https://picoflash.org), which has support for viewing OTP data in both ECC and raw modes.

## Hardware

Primarily outside the scope of this document and library, but interesting nonetheless ...

OTP uses antifuse bit cells, which store data as a charge, similar to a flash bit cell.  This is in constrast to traditional fuse bit cells, which store data as a physical connection.

It also means that it is harder to extract OTP data by physical means, as the charge is harder to detect than a physical connection, although there is a technique to achieve this.

Bits are paired - with pairs being from row 0/32, 1/33, etc within a page.  This can make it harder to physically extract OTP data using the above technique, as it is difficult to distinguish between the paired "bit cells".

## Basic Access

Access to OTP is via three mechanisms:

- Memory mapped access (0x40134000 for raw mode - 4 bytes per row, and 0x4013000 for ECC mode - 2 bytes per row)
- Using a bootrom API otp_access (which, presumably, uses memory mapped access internally)
- Using the PICOBOOT protocol (using the bootrom API internally)

As well as accessing OTP registers via code running on the RP2350, OTP can also be accessed using memory addresses, via the debug interface, using the SWD and JTAG.  There also appears to be a dedicated OTP AP in the debug interface.

See [Protection and Security](#protection-and-security) for more details of access protection and security.

When using memory mapped access, a 32-bit read on the ECC region returns a pair of adjacent rows

There is some complexity around specific memory mapped regions - see the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) for more details.

## User Data

Pages 3 through 60 (inclusive) are available for user data.  These are rows 0xC0 through 0xF3F (inclusive).

There are around 3,712 free rows available for user data, including uses such as USB identification.  This equates to 7,424 bytes (using ECC mode).

Page 2, 0x080 - 0x0bf, is available if secure boot is disabled, and a partial set is available for lesser secure boot configurations.

Pages 0, 1 and 61-63 are reserved for Raspberry Pi use.  Obviously, using free rows in this range is likely to work with current hardware, but may break with future RP2350 steppings. 

## Protection and Security

Pages can be marked as read-only or inaccessible, for different types of access (secure, non-secure and bootloader).

This locking is done using specific OTP rows, so, like other OTP data, once locked, it cannot be unlocked.  Locking therefore goes from most permissive to least permissive - read/write -> read-only -> inaccessible.

(There is also soft locking achieved using CPU registers, but this is not persistent across resets and power cycles.)

OTP also supports key access to pages, where keys are written to OTP locations, and a CPU register is written to to provide access to those pages.

Page 0 is locked as read-only during factory testing, and cannot be unlocked.  This contains serial number, factory calibration data, and other critical information.

Note that there are some errata in the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) regarding OTP protection and security.  See the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) for more details.

## Boot Access

OTP is read as one of the first operations performed at boot time, after the cold reset is removed from both processors.  This is because OTP is used for boot security hardware information, and bootloader configuration.

## Errata

There are a number of [errata](#errata) listed in the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) with respect to OTP.

### RP2350-E17

The most notable for this library is [E17](#rp2350-e17) and essentially means that ECC checking is done simultaneously for each adjacent even/odd pair, rather than each row individually.

This means avoiding:
- storing ECC and raw data in adjacent pairs of rows (starting on the even row)
- storing two sets of ECC data in adjacent pairs of rows (starting on the even row) when that data is protected by difference enable flags elsewhere in OTP.

There are some pre-existing ECC rows that violate this rule - see the [datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) for details.

## Hardware Flaws/Previous Programming

The ECC mode supports writing ECC data to a row with up to a single bit already set to 1, either due to manufacturing defects, or previous programming.

If this is experienced, the low two bytes of the row when read using raw mode will contain the original data, with the additional high bit, not the corrected data.

As an anecdote, reviewing a couple of different RP2350 devices, no rows that were expected to be empty were found to have any bits set to 1, when queried in raw mode.
