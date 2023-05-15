// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use proto::digital_twin::digital_twin_client::DigitalTwinClient;
use proto::digital_twin::FindByIdRequest;
use samples_common::{digital_twin_operation, digital_twin_protocol, is_subset};
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_proto::sample_grpc::v1::digital_twin_provider::SubscribeRequest;
use std::net::SocketAddr;
use tonic::transport::Server;

mod consumer_impl;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138

const CONSUMER_AUTHORITY: &str = "[::1]:60010";

/// Get the provider URI.
///
/// # Arguments
/// `entity_id` - The matching entity id.
/// `protocol` - The required protocol.
/// `operations` - The required operations.
async fn get_provider_uri(
    entity_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<String, String> {
    info!("Sending a find_by_id request for entity id {entity_id} to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI)
        .await
        .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByIdRequest { id: entity_id.to_string() });
    let response = client.find_by_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    let entity_access_info = response_inner.entity_access_info.expect("Did not find the entity");

    let mut provider_uri_option: Option<String> = None;
    for endpoint_info in entity_access_info.endpoint_info_list {
        // We require and endpoint that supports the protocol and supports all of the operations.
        if endpoint_info.protocol == protocol
            && is_subset(operations, endpoint_info.operations.as_slice())
        {
            provider_uri_option = Some(endpoint_info.uri);
            break;
        }
    }

    if provider_uri_option.is_none() {
        return Err("Did not find an endpoint that met our requirements".to_string());
    }

    let provider_uri = provider_uri_option.unwrap();

    info!("The provider URI for entity id {entity_id} is {provider_uri}");

    Ok(provider_uri)
}

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

    let provider_uri = get_provider_uri(
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::SUBSCRIBE.to_string()],
    )
    .await
    .unwrap();

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
