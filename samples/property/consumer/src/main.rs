// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_url, retry_async_based_on_status};
use samples_common::consumer_config;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::SubscribeRequest;
use std::net::SocketAddr;
use tokio::time::Duration;
use tonic::{Status, transport::Server};

/// Subscribe to the ambient air temperature.
///
/// # Arguments
/// * `provider_uri` - The provider's URI.
/// * `consumer_uri` - The consumer's URI.
async fn subscribe_to_ambient_air_temperature(
    provider_uri: &str,
    consumer_uri: &str,
) -> Result<(), Status> {
    let mut client = DigitalTwinProviderClient::connect(provider_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        consumer_uri: consumer_uri.to_string(),
    });
    let _response = client.subscribe(request).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    let settings = consumer_config::load_settings();

    let invehicle_digital_twin_url = retrieve_invehicle_digital_twin_url(
        settings.invehicle_digital_twin_url,
        settings.chariott_url,
    )
    .await?;

    let consumer_authority = settings.consumer_authority;

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse().unwrap();
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_url,
        sdv::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::SUBSCRIBE.to_string()],
    )
    .await
    .unwrap();
    let provider_uri = provider_endpoint_info.uri;
    info!("The URI for the AmbientAirTemperature property's provider is {provider_uri}");

    // Construct the consumer URI from the consumer authority.
    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    info!(
        "Sending a subscribe request for entity id {} to provider URI {provider_uri}",
        sdv::hvac::ambient_air_temperature::ID
    );
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        subscribe_to_ambient_air_temperature(&provider_uri, &consumer_uri)
    })
    .await?;

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
