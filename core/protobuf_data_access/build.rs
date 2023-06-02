// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .message_attribute("EndpointInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("EntityAccessInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(
            &["../../interfaces/digital_twin/v1/digital_twin.proto"],
            &["../../interfaces/digital_twin/v1/"],
        )?;
    tonic_build::configure().compile(
        &["../../external/chariott/proto/chariott/runtime/v1/runtime.proto"],
        &["../../external/chariott/proto"],
    )?;
    tonic_build::configure().compile(
        &["../../external/chariott/proto/chariott/provider/v1/provider.proto"],
        &["../../external/chariott/proto"],
    )?;

    Ok(())
}
