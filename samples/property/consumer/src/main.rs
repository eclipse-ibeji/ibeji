// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use samples_common::{digital_twin_operation, digital_twin_protocol, find_provider_endpoint};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::SubscribeRequest;
use std::net::SocketAddr;
use tonic::transport::Server;

mod consumer_impl;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138

const CONSUMER_AUTHORITY: &str = "[::1]:60010";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = CONSUMER_AUTHORITY.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{CONSUMER_AUTHORITY}'");

    let provider_endpoint_info = find_provider_endpoint(
        IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI,
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::SUBSCRIBE.to_string()],
    )
    .await
    .unwrap();

    let provider_uri = provider_endpoint_info.uri;

    info!("The URI for the AmbientAirTemperature property's provider is {provider_uri}");

    let consumer_uri = format!("http://{CONSUMER_AUTHORITY}"); // Devskim: ignore DS137138

    // Subscribing to the ambient air temperature data feed.
    info!(
        "Sending a subscribe request for entity id {} to provider URI {provider_uri}",
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID
    );
    let mut client = DigitalTwinProviderClient::connect(provider_uri).await?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: sdv::vehicle::cabin::hvac::ambient_air_temperature::ID.to_string(),
        consumer_uri,
    });
    let _response = client.subscribe(request).await?;

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
