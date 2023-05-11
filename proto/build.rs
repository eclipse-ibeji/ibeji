// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .message_attribute("EndpointInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("EntityAccessInfo", "#[derive(serde::Deserialize, serde::Serialize)]")        
        .compile(&["digital_twin/v1/digital_twin.proto"], &["digital_twin/v1/"])?;

    Ok(())
}
