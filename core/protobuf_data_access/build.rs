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
    tonic_build::configure()
        .message_attribute("Constraint", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("CallbackPayload", "#[derive(serde::Deserialize, serde::Serialize)]")
        .message_attribute("SubscriptionInfo", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(
            &["../../interfaces/extension/managed_subscribe/v1/managed_subscribe.proto"],
            &["../../interfaces/extension/managed_subscribe/v1/"],
        )?;
    tonic_build::configure().compile(
        &["../../external/chariott/service_discovery/proto/core/v1/service_registry.proto"],
        &["../../external/chariott/service_discovery/proto/core/v1/"],
    )?;
    tonic_build::configure().compile(
        &["../../external/agemo/proto/pubsub/v1/pubsub.proto"],
        &["../../external/agemo/proto/pubsub/v1/"],
    )?;
    tonic_build::configure().compile(
        &["../../external/agemo/proto/publisher/v1/publisher.proto"],
        &["../../external/agemo/proto/publisher/v1/"],
    )?;

    Ok(())
}
