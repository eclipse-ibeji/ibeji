// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../interfaces/sample_grpc/v1/digital_twin_consumer.proto")?;
    tonic_build::compile_protos("../interfaces/sample_grpc/v1/digital_twin_provider.proto")?;
    tonic_build::configure()
        .message_attribute("EndpointInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("EntityAccessInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(
            &["../../interfaces/digital_twin/v1/digital_twin.proto"],
            &["../../interfaces/digital_twin/v1/"],
        )?;

    Ok(())
}
