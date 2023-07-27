// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![cfg(feature = "yaml")]

use config::{Config, File, FileFormat};
use serde_derive::Deserialize;

const CONFIG_FILENAME: &str = "provider_settings";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub provider_authority: String,
    pub chariott_uri: Option<String>,
    pub invehicle_digital_twin_uri: Option<String>,
}

/// Load the settings.
pub fn load_settings() -> Settings {
    let config =
        Config::builder().add_source(File::new(CONFIG_FILENAME, FileFormat::Yaml)).build().unwrap();

    let settings: Settings = config.try_deserialize().unwrap();

    settings
}
