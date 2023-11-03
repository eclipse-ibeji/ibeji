// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::utils;

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
    utils::load_settings(CONFIG_FILENAME).unwrap()
}
