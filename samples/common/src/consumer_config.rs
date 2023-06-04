// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![cfg(feature = "yaml")]

use config::{Config, File, FileFormat};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub consumer_authority: String,
    pub chariott_url: Option<String>,
    pub invehicle_digital_twin_url: Option<String>,
}

/// Load the settings.
pub fn load_settings() -> Settings {
    let config = Config::builder()
        .add_source(File::new("consumer_settings", FileFormat::Yaml))
        .build()
        .unwrap();

    let settings: Settings = config.try_deserialize().unwrap();

    settings
}
