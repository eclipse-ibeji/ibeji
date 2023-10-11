// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// This lib.rs is for running the .NET unit tests in this dtdl-tools folder.
// Running 'cargo test' will run all the .NET unit tests and the Rust unit tests.
#[cfg(test)]
mod dtdl_tools_tests {
    use std::env;
    use std::io::{self, Write};
    use std::path::Path;
    use std::process::Command;

    // The out directory is the directory that contains the build artifacts.
    const OUT_DIR: &str = env!("OUT_DIR");

    const DTDL_VALIDATOR_FILENAME: &str = "dtdl-validator";
    const DTDL_VALIDATOR_BIN_DIR: &str = "dtdl-validator/bin/Debug/net7.0";
    const DTDL_VALIDATOR_EXT_OPTION: &str = "-e";

    /// Validate DTDL files.
    ///
    /// # Arguments
    /// * `directory` - The directory that contains the DTDL files that you wish to validate.
    /// * `extension` - The file extension that the DTDL files use.
    fn validate_dtdl_files(directory: &str, extension: &str) -> bool {
        let dtdl_validator_command_path =
            Path::new(OUT_DIR).join(DTDL_VALIDATOR_BIN_DIR).join(DTDL_VALIDATOR_FILENAME);

        let dtdl_validator_output = Command::new(dtdl_validator_command_path)
            .arg(directory)
            .arg(DTDL_VALIDATOR_EXT_OPTION)
            .arg(extension)
            .output()
            .unwrap();

        if !dtdl_validator_output.status.success() {
            io::stdout().write_all(&dtdl_validator_output.stdout).unwrap();
        }

        dtdl_validator_output.status.success()
    }

    #[test]
    fn validate_digital_twin_model_dtdl_files() {
        assert!(validate_dtdl_files("../digital-twin-model/dtdl", "json"));
    }
}
