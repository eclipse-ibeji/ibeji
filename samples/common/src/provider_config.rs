// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::utils;

use serde_derive::Deserialize;

const DEFAULT_CONFIG_FILENAME: &str = "provider_settings";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub provider_authority: String,
    pub chariott_uri: Option<String>,
    pub invehicle_digital_twin_uri: Option<String>,
}

/// Load the settings using the default config filename.
pub fn load_settings() -> Settings {
    load_settings_with_config_filename(DEFAULT_CONFIG_FILENAME)
}

// Load the settings using the specified config filename.
pub fn load_settings_with_config_filename(config_filename: &str) -> Settings {
    utils::load_settings(config_filename).unwrap()
}
