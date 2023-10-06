// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// This lib.rs is for running the .NET unit tests in this digital_twins_connector folder.
// Running 'cargo test' will run all the .NET unit tests and the Rust unit tests.
#[cfg(test)]
mod digital_twins_connector_dotnet_tests {
    use std::io::{self, Write};
    use std::path::Path;    
    use std::process::Command;

    // The manifest directory is the directory that contains the Cargo.toml file for this crate.
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    const DTDL_VALIDATOR_FILENAME: &str = "dtdl-validator";
    const DTDL_VALIDATOR_BIN_DIR: &str = "target/debug/dtdl-validator/bin/Debug/net7.0";
    const DTDL_VALIDATOR_DIR_OPTION: &str = "-d";
    const DTDL_VALIDATOR_EXT_OPTION: &str = "-e";  

    /// Validate DTDL files.
    /// 
    /// # Arguments
    /// * `directory` - The directory that contains the DTDL files that you wish to validate.
    /// * `extension` - The file extension that the DTDL files use.
    fn validate_dtdl_files(directory: &str, extension: &str) -> bool {

        let dtdl_validator_command_path = Path::new(MANIFEST_DIR).join("..").join(DTDL_VALIDATOR_BIN_DIR).join(DTDL_VALIDATOR_FILENAME);
    
        let dtdl_validator_output = Command::new(dtdl_validator_command_path)
            .arg(DTDL_VALIDATOR_DIR_OPTION)
            .arg(directory)
            .arg(DTDL_VALIDATOR_EXT_OPTION)
            .arg(extension)            
            .output()
            .unwrap();

        if !dtdl_validator_output.status.success() {
            io::stdout().write_all(&dtdl_validator_output.stdout).unwrap();
        }

        return dtdl_validator_output.status.success();
    }

    #[test]
    fn validate_digital_twin_model_dtdl_files() {
        assert!(validate_dtdl_files("../digital-twin-model/dtdl", "json"));
    }
}