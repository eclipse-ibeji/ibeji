// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .message_attribute("EndpointInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("EntityAccessInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(
            &["../../interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto"],
            &["../../interfaces/invehicle_digital_twin/v1/"],
        )?;
    tonic_build::configure().compile(
        &["../../external/chariott/service_discovery/proto/core/v1/service_registry.proto"],
        &["../../external/chariott/service_discovery/proto/core/v1/"],
    )?;

    Ok(())
}
