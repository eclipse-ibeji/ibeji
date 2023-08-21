// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_config;
mod provider_impl;

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::Duration;
use tonic::{Status, transport::Server};

use crate::provider_impl::ProviderImpl;

#[derive(Debug, Serialize, Deserialize)]
struct AmbientAirTemperatureProperty {
    #[serde(rename = "AmbientAirTemperature")]
    ambient_air_temperature: sdv::hvac::ambient_air_temperature::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct HybridBatteryRemainingProperty {
    #[serde(rename = "HybridBatteryRemainaing")]
    hybrid_battery_remaining: sdv::obd::hybrid_battery_remaining::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct IsAirConditioingActiveProperty {
    #[serde(rename = "IsAirConditioingActive")]
    is_air_conditioning_active: sdv::hvac::is_air_conditioning_active::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Register the entities endpoints.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
async fn register_entities(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    // Camera Feed
    let camera_feed_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::STREAM.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::camera::feed::ID.to_string(),
    };
    let camera_feed_access_info = EntityAccessInfo {
        name: sdv::camera::feed::NAME.to_string(),
        id: sdv::camera::feed::ID.to_string(),
        description: sdv::camera::feed::DESCRIPTION.to_string(),
        endpoint_info_list: vec![camera_feed_endpoint_info],
    };

    let entity_access_info_list = vec![camera_feed_access_info];

    println!("Registering the list {:?}", entity_access_info_list);

    let mut client = InvehicleDigitalTwinClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(RegisterRequest { entity_access_info_list });
    let _response = client.register(request).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = crate::provider_config::load_settings();

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    // Construct the provider URI from the provider authority.
    let provider_uri = format!("http://{provider_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = provider_authority.parse()?;
    let provider_impl = ProviderImpl::new(&settings.image_directory);
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_entities(&invehicle_digital_twin_uri, &provider_uri)
    })
    .await?;

    server_future.await?;

    info!("The Provider has completed.");

    Ok(())
}
