// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::misc::{discover_digital_twin_service_using_chariott, discover_digital_twin_provider_using_ibeji};
use samples_common::consumer_config::load_settings;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::SubscribeRequest;
use std::net::SocketAddr;
use tonic::transport::Server;

mod consumer_impl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    let settings = load_settings();

    let invehicle_digital_twin_url = match settings.invehicle_digital_twin_url {
        Some(value) => value,
        None => {
            match settings.chariott_url {
                Some(value) => {
                    match discover_digital_twin_service_using_chariott(&value).await {
                        Ok(Some(value)) => value,
                        Ok(None) => Err("Failed to discover the in-vehicle digital twin service's URL, as it is not registered with Chariott")?,
                        Err(error) => Err(format!("Failed to discover the in-vehicle digital twin service's URL due to error: {error}"))?
                    }                    
                }
                None => {
                    Err("The settings file must set a chariott_url setting when the invehicle_digital_twin_url is not set.")?
                }          
            }
        }
    };

    // Setup the HTTP server.
    let consumer_authority = settings.consumer_authority;
    let addr: SocketAddr = consumer_authority.parse().unwrap();
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    // Workarounhd: see https://stackoverflow.com/questions/23975391/how-to-convert-a-string-into-a-static-str
    let static_url_str = Box::leak(invehicle_digital_twin_url.into_boxed_str());

    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        static_url_str,
        sdv::vehicle::cabin::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::SUBSCRIBE.to_string()],
    )
    .await
    .unwrap();

    let provider_uri = provider_endpoint_info.uri;

    info!("The URI for the AmbientAirTemperature property's provider is {provider_uri}");

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

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
