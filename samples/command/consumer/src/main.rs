// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use samples_common::{digital_twin_operation, digital_twin_protocol, find_provider_endpoint};
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_proto::sample_grpc::v1::digital_twin_provider::InvokeRequest;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const CONSUMER_AUTHORITY: &str = "[::1]:60010";

/// Start the show notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_show_notification_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the Consumer's show notification repeater.");
    tokio::spawn(async move {
        loop {
            let payload: String = "The show-notification request.".to_string();

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
                entity_id: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID
                    .to_string(),
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
    let addr: SocketAddr = CONSUMER_AUTHORITY.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{CONSUMER_AUTHORITY}'");

    let provider_endpoint_info = find_provider_endpoint(
        IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI,
        sdv::vehicle::cabin::infotainment::hmi::show_notification::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::INVOKE.to_string()],
    )
    .await
    .unwrap();

    let provider_uri = provider_endpoint_info.uri;

    info!("The URI for the ShowNotification command's provider is {provider_uri}");

    let consumer_uri = format!("http://{CONSUMER_AUTHORITY}"); // Devskim: ignore DS137138

    start_show_notification_repeater(provider_uri, consumer_uri);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
