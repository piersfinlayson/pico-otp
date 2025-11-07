// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// Allow process_wl_schema() to be unused as it's only used temporarily to
// hand-generate a new `src/whitelabel.rs` from the schema JSON.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

const WL_SCHEMA_PATH: &str = "json/whitelabel-schema.json";
const WL_JSON_FILE: &str = "whitelabel.rs";

fn main() {
    // Re-run if the schema file changes
    println!("cargo:rerun-if-changed={}", WL_SCHEMA_PATH);

    // Process the whitelabel schema into Rust types
    // process_wl_schema();
}

fn process_wl_schema() {
    let schema = fs::read_to_string(WL_SCHEMA_PATH).unwrap();

    let mut type_space = typify::TypeSpace::default();
    type_space
        .add_root_schema(serde_json::from_str(&schema).unwrap())
        .unwrap();

    let contents = format!(
        "{}",
        prettyplease::unparse(&syn::parse2(type_space.to_stream()).unwrap())
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join(WL_JSON_FILE), contents).unwrap();
}
