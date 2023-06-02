// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::misc::{retrieve_invehicle_digital_twin_url, retry_async_based_on_status};
use samples_common::provider_config::load_settings;
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use tonic::{Status, transport::Server};

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

/// Register the show notification commans's endpoint.
///
/// # Arguments
/// * `invehicle_digital_twin_url` - The In-Vehicle Digital Twin URL.
/// * `provider_uri` - The provider's URI.
async fn register_show_notification(
    invehicle_digital_twin_url: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::INVOKE.to_string()],
        uri: provider_uri.to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: "ShowNotification".to_string(),
        id: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
        description: "Show a notification on the HMI.".to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    let mut client = DigitalTwinClient::connect(invehicle_digital_twin_url.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;

    Ok(())
}

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = load_settings();

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_url = retrieve_invehicle_digital_twin_url(
        settings.invehicle_digital_twin_url,
        settings.chariott_url,
    )
    .await?;

    // Construct the provider URI from the provider authority.
    let provider_uri = format!("http://{provider_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = provider_authority.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URL {invehicle_digital_twin_url}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_show_notification(&invehicle_digital_twin_url, &provider_uri)
    })
    .await?;

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
