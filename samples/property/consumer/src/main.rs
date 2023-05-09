// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use dt_model_identifiers::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::FindByIdRequest;
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_proto::sample_grpc::v1::digital_twin_provider::SubscribeRequest;
use std::net::SocketAddr;
use tonic::transport::Server;

mod consumer_impl;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138

const CONSUMER_ADDR: &str = "[::1]:60010";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = CONSUMER_ADDR.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);

    // Obtain the DTDL for the ambient air temmpterature.
    info!("Sending a find_by_id request for entity id {} to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}",
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID);
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request = tonic::Request::new(FindByIdRequest {
        id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
    });
    let response = client.find_by_id(request).await?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    let provider_uri = match response_inner.entity_access_info {
        Some(content) => {
            // TODO: select the right one, rather than just using the first one
            content.endpoint_info_list[0].uri.clone()
        },
        None => {
            panic!("Did not find an entity for the AmbientAirTemperature command");
        }
    };

    let consumer_uri = format!("http://{CONSUMER_ADDR}"); // Devskim: ignore DS137138

    // Subscribing to the ambient air temperature data feed.
    info!(
        "Sending a subscribe request for entity id {} to provider URI {provider_uri}",
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID
    );
    let mut client = DigitalTwinProviderClient::connect(provider_uri).await?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: String::from(sdv::vehicle::cabin::hvac::ambient_air_temperature::ID),
        consumer_uri,
    });
    let _response = client.subscribe(request).await?;

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
