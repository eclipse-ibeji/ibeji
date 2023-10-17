// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::env;
use std::io::{self, Write};
use std::process::Command;

// This build script builds all the .NET projects in this dtdl-tools folder.
// Running 'cargo build' will build the .NET projects and the Rust crates.
fn main() {
    const DOTNET_COMMAND: &str = "dotnet";
    const DOTNET_BUILD_ARG: &str = "build";
    const CSPROJ_PATHS: &[&str] = &["src/dtdl-validator/dtdl-validator.csproj"];
    const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    for csproj in CSPROJ_PATHS {
        let output =
            Command::new(DOTNET_COMMAND).arg(DOTNET_BUILD_ARG).arg(csproj).output().unwrap();

        if !output.status.success() {
            panic!("Failed to run due to {output:?}");
        }

        io::stdout().write_all(&output.stdout).unwrap();
    }

    println!("cargo:rerun-if-changed={CARGO_MANIFEST_DIR}/src/dtdl-validator");
}
