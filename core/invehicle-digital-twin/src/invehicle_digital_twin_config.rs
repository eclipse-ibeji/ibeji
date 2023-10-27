// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use common::utils;
use serde_derive::Deserialize;

const CONFIG_FILENAME: &str = "invehicle_digital_twin_settings";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub invehicle_digital_twin_authority: String,
    pub chariott_uri: Option<String>,
}

/// Load the settings.
pub fn load_settings() -> Settings {
    let settings = utils::load_settings(CONFIG_FILENAME).unwrap();

    settings
}
