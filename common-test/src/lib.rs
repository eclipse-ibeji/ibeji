// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::{trace, warn};
use std::env;
use std::path::Path;

/// The DTDL-path environment variable name.
pub const DTDL_PATH: &str = "DTDL_PATH";

/// The DTDL-path environment variable name.
pub const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

/// Get the repository's directory.
fn get_repo_dir() -> Option<String> {
    // CARGO_MANIFEST_DIR - The directory containing the manifest of your package.
    let cargo_manifest_dir_result = env::var(CARGO_MANIFEST_DIR);
    if let Ok(cargo_manifest_dir) = cargo_manifest_dir_result {
        let cargo_manifest_dir_path = Path::new(&cargo_manifest_dir);
        let parent_result = cargo_manifest_dir_path.parent();
        if let Some(parent) = parent_result {
            parent.to_str().map(String::from)
        } else {
            None
        }
    } else {
        None
    }
}

/// Set the DTDL_PATH environment, so that the tests can use it.
pub fn set_dtdl_path() {
    let repo_dir_result = get_repo_dir();
    if let Some(repo_dir) = repo_dir_result {
        let value = format!(
            "{}/opendigitaltwins-dtdl/DTDL;{}/iot-plugandplay-models;{}/dtdl",
            repo_dir, repo_dir, repo_dir
        );
        env::set_var(DTDL_PATH, &value);
        trace!("{}={}", DTDL_PATH, &value);
    } else {
        warn!("Unable to set {}, as repo directory could not be determined.", DTDL_PATH);
    }
}

#[cfg(test)]
mod ibeji_common_test_tests {
    use super::*;
    use std::env;

    #[test]
    fn find_by_id_test() {
        env::remove_var(DTDL_PATH);
        set_dtdl_path();
        let get_dtdl_path_result = env::var(DTDL_PATH);
        assert!(get_dtdl_path_result.is_ok());
        let dtdl_path = get_dtdl_path_result.unwrap();
        assert!(!dtdl_path.is_empty());
        assert!(dtdl_path.contains("/opendigitaltwins-dtdl/DTDL;"));
        assert!(dtdl_path.contains("/dtdl"));
    }
}
