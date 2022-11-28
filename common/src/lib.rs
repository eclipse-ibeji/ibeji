// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::env;
use std::fs;
use std::path::Path;

/// The DTDL-path environment variable name.
pub const DTDL_PATH: &str = "DTDL_PATH";

/// Find the full path given a relative path and a preset DTDL_PATH environment variable (containing a semicolon-separated list of DTDL directories).
///
/// # Arguments
/// `relative_path` - The relative path.
pub fn find_full_path(relative_path: &str) -> Result<String, String> {
    match env::var(DTDL_PATH) {
        Ok(paths) => {
            let split = paths.split(';');
            let vec: Vec<&str> = split.collect();
            for path in vec {
                let full_path = Path::new(path).join(relative_path);
                if full_path.exists() {
                    return Ok(full_path.to_str().unwrap().to_string());
                }
            }
        }
        Err(_) => {
            return Err(format!(
                "Unable to get the environment variable {}. Please set it.",
                DTDL_PATH
            ))
        }
    }
    Err(String::from("Unable to resolve the full path"))
}

/// Retrieve the contents of the DTDL from the specified file path.
///
/// # Arguments:
/// `file_path` - The file path where the DTDL is located.
pub fn retrieve_dtdl(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => Ok(contents),
        Err(error) => Err(format!("Unable to retrieve the DTDL due to: {:?}", error)),
    }
}

#[cfg(test)]
mod ibeji_common_tests {
    use super::*;
    use ibeji_common_test::set_dtdl_path;

    #[test]
    fn find_full_path_test() {
        set_dtdl_path();

        let find_full_path_result = find_full_path("samples/remotely_accessible_resource.json");
        assert!(find_full_path_result.is_ok());
        let full_path = find_full_path_result.unwrap();
        assert!(full_path.ends_with("/samples/remotely_accessible_resource.json"));
    }
}
