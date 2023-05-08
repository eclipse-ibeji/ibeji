// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use dt_model_identifiers::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::FindByIdRequest;
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_proto::sample_grpc::v1::digital_twin_provider::InvokeRequest;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const CONSUMER_ADDR: &str = "[::1]:60010";

/// Start the show notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_show_notification_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the Consumer's show notification repeater.");
    tokio::spawn(async move {
        loop {
            let payload: String = String::from("The show-notification request.");

            info!("Sending an invoke request on entity {} with payload '{payload} to provider URI {provider_uri}",
                sdv::vehicle::cabin::infotainment::hmi::show_notification::ID);

            let client_result = DigitalTwinProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let request = tonic::Request::new(InvokeRequest {
                entity_id: String::from(
                    sdv::vehicle::cabin::infotainment::hmi::show_notification::ID,
                ),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload,
            });

            let response = client.invoke(request).await;
            match response {
                Ok(_) => (),
                Err(status) => warn!("{status:?}"),
            }

            debug!("Invoked the show-notification command on endpoint {provider_uri}");

            sleep(Duration::from_secs(5)).await;
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    // Setup the HTTP server.
    let consumer_authority = String::from(CONSUMER_ADDR);
    let addr: SocketAddr = consumer_authority.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{CONSUMER_ADDR}'");

    info!("Sending a find_by_id request for entity id {} to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}",
        sdv::vehicle::cabin::infotainment::hmi::show_notification::ID);
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request = tonic::Request::new(FindByIdRequest {
        id: String::from(sdv::vehicle::cabin::infotainment::hmi::show_notification::ID),
    });
    let response = client.find_by_id(request).await?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    let provider_uri;
    match response_inner.entity_access_info {
        Some(content) => {
            // TODO: select the right one, rather than just using the first one
            provider_uri = content.endpoint_info_list[0].uri.clone();
        }
        None => {
            panic!("Did not find an entity for the ShowNotification command");
        }
    }

    info!("The URI for the show-notification command's provider is {provider_uri}");

    let consumer_uri = format!("http://{CONSUMER_ADDR}"); // Devskim: ignore DS137138

    start_show_notification_repeater(provider_uri, consumer_uri);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
