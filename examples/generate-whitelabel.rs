// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! Generates Rust types from `json/whitelabel-schema.json` and prints them to
//! stdout.
//!
//! `src/whitelabel/auto/mod.rs` began as this output. Its header comment says
//! how to prepare the schema first and what to edit afterwards.
//!
//! ```sh
//! cargo run --example generate-whitelabel
//! ```

use std::fs;

const WL_SCHEMA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/json/whitelabel-schema.json");

fn main() {
    let schema = fs::read_to_string(WL_SCHEMA_PATH).unwrap();

    let mut type_space = typify::TypeSpace::default();
    type_space
        .add_root_schema(serde_json::from_str(&schema).unwrap())
        .unwrap();

    let contents = prettyplease::unparse(&syn::parse2(type_space.to_stream()).unwrap());
    print!("{contents}");
}
