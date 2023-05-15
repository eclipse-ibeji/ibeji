// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use proto::digital_twin::digital_twin_client::DigitalTwinClient;
use proto::digital_twin::FindByIdRequest;
use samples_common::{digital_twin_operation, digital_twin_protocol, is_subset};
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_proto::sample_grpc::v1::digital_twin_provider::InvokeRequest;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

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
        if endpoint_info.protocol == protocol && is_subset(operations, endpoint_info.operations.as_slice()) {
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

    let provider_uri = get_provider_uri(
        sdv::vehicle::cabin::infotainment::hmi::show_notification::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::INVOKE.to_string()],
    )
    .await
    .unwrap();

    info!("The URI for the ShowNotification command's provider is {provider_uri}");

    let consumer_uri = format!("http://{CONSUMER_AUTHORITY}"); // Devskim: ignore DS137138

    start_show_notification_repeater(provider_uri, consumer_uri);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
