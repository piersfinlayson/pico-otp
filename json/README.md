# JSON Schema

[`whitelabel-schema.json`](whitelabel-schema.json) was downloaded from: https://raw.githubusercontent.com/raspberrypi/picotool/develop/json/schemas/whitelabel-schema.json

And then modified to remove "pattern" fields, as parsing those with `typify` requires `std`:
- `device:properties:vid` - "^0x[0-9a-fA-F]{4}$"
- `device:properties:pid` - "^0x[0-9a-fA-F]{4}$"
- `device:properties:lang_id` - "^0x[0-9a-fA-F]{4}$"
- `device:properties:max_power` - "^0x[0-9a-fA-F]{1,2}$"
- `device:properties:attributes` - "^0x[8aceACE]{1}0$"

These are hand validated in Rust instead, on `/src/whitelabel/mod.rs`.

The original is used as the JSON schema for defining USB white label data in `pico-otp`.

It is also used by Raspberry Pi's [picotool](https://github.com/raspberrypi/picotool), which is licensed as described [here](LICENSE-SCHEMA.md).
