// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use common::utils;
use serde_derive::Deserialize;

const DEFAULT_CONFIG_FILENAME: &str = "digital_twin_graph_settings";

/// The settings for the digital twin graph service.
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// The authority (address + optional port in the format "<address>[:<port>]") for the Ibeji application server.
    pub base_authority: String,
}

/// Load the settings.
/// The settings are loaded from the default config file name.
///
/// # Returns
/// The settings.
pub fn load_settings() -> Settings {
    utils::load_settings(DEFAULT_CONFIG_FILENAME).unwrap()
}

/// Load the settings with the specified config file name.
///
/// # Arguments
/// * `config_filename` - The name of the config file.
/// # Returns
/// The settings.
pub fn load_settings_with_config_filename(config_filename: &str) -> Settings {
    utils::load_settings(config_filename).unwrap()
}
