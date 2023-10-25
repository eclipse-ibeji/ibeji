// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::env;
use config::{Config, File, FileFormat};
use serde_derive::Deserialize;

const CONFIG_FILENAME: &str = "invehicle_digital_twin_settings";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub invehicle_digital_twin_authority: String,
    pub chariott_uri: Option<String>,
}

/// Load the settings.
pub fn load_settings() -> Settings {
    let config_filename_path = match env::var("IBEJI_HOME") {
        Ok(s) => format!("{}/{}", s, CONFIG_FILENAME),
        _ => CONFIG_FILENAME.to_owned()
    };
   
    let config =
        Config::builder().add_source(File::new(&config_filename_path, FileFormat::Yaml)).build().unwrap();

    let settings: Settings = config.try_deserialize().unwrap();

    settings
}
