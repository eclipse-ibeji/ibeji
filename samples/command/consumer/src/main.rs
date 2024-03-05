// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::{sdv_v0 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::consumer_config;
use samples_common::utils::{discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::InvokeRequest;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::transport::Server;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ShowNotificationRequestPayload {
    #[serde(rename = "Notification")]
    notification: sdv::hmi::show_notification::request::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Start the show notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_show_notification_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the Consumer's show notification repeater.");

    let request_payload: ShowNotificationRequestPayload = ShowNotificationRequestPayload {
        notification: "The show-notification request.".to_string(),
        metadata: Metadata { model: sdv::hmi::show_notification::request::ID.to_string() },
    };

    let request_payload_json = serde_json::to_string(&request_payload).unwrap();

    tokio::spawn(async move {
        loop {
            info!("Sending an invoke request on entity {} with payload '{}' to provider URI {provider_uri}",
                sdv::hmi::show_notification::ID, &request_payload_json);

            let client_result = DigitalTwinProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let response_id = Uuid::new_v4().to_string();

            let request = tonic::Request::new(InvokeRequest {
                entity_id: sdv::hmi::show_notification::ID.to_string(),
                consumer_uri: consumer_uri.clone(),
                response_id,
                payload: request_payload_json.to_string(),
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

    let settings = consumer_config::load_settings();

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    let consumer_authority = settings
        .consumer_authority
        .expect("consumer_authority must be specified in the config file");

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::hmi::show_notification::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::INVOKE.to_string()],
    )
    .await
    .unwrap();

    let provider_uri = provider_endpoint_info.uri;

    info!("The URI for the ShowNotification command's provider is {provider_uri}");
    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    start_show_notification_repeater(provider_uri, consumer_uri);

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
