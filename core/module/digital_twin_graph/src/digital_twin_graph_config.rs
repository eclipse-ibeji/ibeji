// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use common::utils;
use serde_derive::Deserialize;

const DEFAULT_CONFIG_FILENAME: &str = "digital_twin_graph_settings";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub base_authority: String,
}

/// Load the settings.
pub fn load_settings() -> Settings {
    utils::load_settings(DEFAULT_CONFIG_FILENAME).unwrap()
}

/// Load the settings.
pub fn load_settings_with_config_filename(config_filename: &str) -> Settings {
    utils::load_settings(config_filename).unwrap()
}
