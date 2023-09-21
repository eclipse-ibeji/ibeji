// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use config::{Config, File, FileFormat};

/// Load the settings.
///
/// # Arguments
/// * `config_filename` - Name of the config file to load settings from.
pub fn load_settings<T>(config_filename: &str) -> T
where
    T: for<'de> serde::Deserialize<'de>,
{
    let config =
        Config::builder().add_source(File::new(config_filename, FileFormat::Yaml)).build().unwrap();

    let settings: T = config.try_deserialize().unwrap();

    settings
}
