// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![cfg(feature = "yaml")]

use serde_derive::Deserialize;
use config::{Config, File, FileFormat};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub provider_authority: String,
    pub chariott_url: Option<String>,
    pub invehicle_digital_twin_url: Option<String>,
}

pub fn load_settings() -> Settings {
    let config = Config::builder()
        .add_source(File::new("provider_settings", FileFormat::Yaml))
        .build()
        .unwrap();

    let settings: Settings = config.try_deserialize().unwrap();

    settings
}
