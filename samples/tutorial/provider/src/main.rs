// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use crate::provider_impl::ProviderImpl;
use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::provider_config;
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{
    EndpointInfo, EntityAccessInfo, RegisterRequest,
};
use samples_protobuf_data_access::tutorial_grpc::v1::digital_twin_provider_tutorial_server::DigitalTwinProviderTutorialServer;
use std::net::SocketAddr;
use tokio::time::Duration;
use tonic::{transport::Server, Status};

/// Register the entities' endpoints with the In-Vehicle Digital Twin Service.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
async fn register_entities(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    // AmbientAirTemperature
    let ambient_air_temperature_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::GET.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::hvac::ambient_air_temperature::ID.to_string(),
    };
    let ambient_air_temperature_access_info = EntityAccessInfo {
        name: sdv::hvac::ambient_air_temperature::NAME.to_string(),
        id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        description: sdv::hvac::ambient_air_temperature::DESCRIPTION.to_string(),
        endpoint_info_list: vec![ambient_air_temperature_endpoint_info],
    };

    // IsAirConditioningActive
    let is_air_conditioning_active_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::GET.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::hvac::is_air_conditioning_active::ID.to_string(),
    };
    let is_air_conditioning_active_access_info = EntityAccessInfo {
        name: sdv::hvac::is_air_conditioning_active::NAME.to_string(),
        id: sdv::hvac::is_air_conditioning_active::ID.to_string(),
        description: sdv::hvac::is_air_conditioning_active::DESCRIPTION.to_string(),
        endpoint_info_list: vec![is_air_conditioning_active_endpoint_info],
    };

    // ShowNotification
    let show_notification_endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::INVOKE.to_string()],
        uri: provider_uri.to_string(),
        context: sdv::hmi::show_notification::ID.to_string(),
    };
    let show_notification_access_info = EntityAccessInfo {
        name: sdv::hmi::show_notification::NAME.to_string(),
        id: sdv::hmi::show_notification::ID.to_string(),
        description: sdv::hmi::show_notification::DESCRIPTION.to_string(),
        endpoint_info_list: vec![show_notification_endpoint_info],
    };

    let entity_access_info_list = vec![
        ambient_air_temperature_access_info,
        is_air_conditioning_active_access_info,
        show_notification_access_info,
    ];

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

    info!("The Digital Twin Provider has started.");

    let settings = provider_config::load_settings();

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
    let provider_impl = ProviderImpl {};
    let server_future = Server::builder()
        .add_service(DigitalTwinProviderTutorialServer::new(provider_impl))
        .serve(addr);
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
