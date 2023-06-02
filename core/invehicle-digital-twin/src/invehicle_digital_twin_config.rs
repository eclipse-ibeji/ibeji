// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![cfg(feature = "yaml")]

use config::{Config, File, FileFormat};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub invehicle_digital_twin_authority: String,
    pub chariott_url: Option<String>,
}

pub fn load_settings() -> Settings {
    let config = Config::builder()
        .add_source(File::new("invehicle_digital_twin_settings", FileFormat::Yaml))
        .build()
        .unwrap();

    let settings: Settings = config.try_deserialize().unwrap();

    settings
}
