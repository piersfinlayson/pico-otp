# Changelog

## [0.2.0] - 2025/11/08

- Significant rework or interface to improve usability.
    - Now use `OtpData::from_json()` to parse JSON directly into OTP data, rather than first creating a `WhiteLabelStruct` and then converting that to `OtpData`.
- Allow JSON file to be created from existing OTP data read from device.
    - Use `OtpData::from_white_label_data()` or "OtpData::from_full_otp_data()" to create `OtpData` from existing OTP data read from device.
    - Use `OtpData::to_json()` to create JSON representation of the OTP data.
- Moved from assert model for consistency checking to returning Error::InternalInconsistency(String).
    - When generating OTP data from JSON, once the JSON is parsed successful, there should be no other errors hit - as the JSON validation is sufficient.
    - When processing existing OTP data read from device, errors may be returned if inconsistencies are found.
    - There are still significant numbers of asserts in the code, but these are now only for conditions that should never occur unless there is a bug in the code.

## [0.1.0] - 2025/11/07

First implementation