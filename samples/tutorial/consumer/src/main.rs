// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::consumer_config;
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
    retry_async_based_on_status,
};
use samples_protobuf_data_access::tutorial_grpc::v1::digital_twin_provider_tutorial_client::DigitalTwinProviderTutorialClient;
use samples_protobuf_data_access::tutorial_grpc::v1::{GetRequest, InvokeRequest};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tonic::Status;

#[derive(Debug, Serialize, Deserialize)]
struct ShowNotificationRequestPayload {
    #[serde(rename = "Notification")]
    notification: sdv::hmi::show_notification::request::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Start the Get Signals Repeater.
/// This will call the get operation on the digital twin provider synchronously to obtain the entity value.
///
/// # Arguments
/// `provider_uri_map` - The provider uri map where the key is the entity id and the value is the provider's uri.
async fn start_get_signals_repeater(
    provider_uri_map: HashMap<String, String>,
) -> Result<(), Status> {
    debug!("Starting the Consumer's get signals repeater.");

    loop {
        for (entity, provider_uri) in &provider_uri_map {
            let response = retry_async_based_on_status(30, Duration::from_secs(1), || {
                send_get_request(provider_uri, entity)
            })
            .await?;

            info!("Get response for entity {entity}: {response}");
        }

        debug!("Completed sending the get requests.");

        sleep(Duration::from_secs(5)).await;
    }
}

/// Start the show-notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
async fn start_show_notification_repeater(provider_uri: String) -> Result<(), Status> {
    debug!("Starting the consumer's show-notification repeater.");

    let metadata = Metadata { model: sdv::hmi::show_notification::request::ID.to_string() };

    let request_payload: ShowNotificationRequestPayload = ShowNotificationRequestPayload {
        notification: "Hello world notification.".to_string(),
        metadata,
    };

    let request_payload_json = serde_json::to_string(&request_payload).unwrap();

    loop {
        info!("Sending an invoke request on entity {} with payload '{}' to provider URI {provider_uri}",
            sdv::hmi::show_notification::ID, &request_payload_json);

        let client_result = DigitalTwinProviderTutorialClient::connect(provider_uri.clone()).await;
        if client_result.is_err() {
            warn!("Unable to connect. We will retry in a moment.");
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        let mut client = client_result.unwrap();

        let request = tonic::Request::new(InvokeRequest {
            entity_id: sdv::hmi::show_notification::ID.to_string(),
            payload: request_payload_json.to_string(),
        });

        let response = client.invoke(request).await?;

        info!("Show notification response: {}", response.into_inner().response);

        debug!("Completed the invoke request");
        sleep(Duration::from_secs(15)).await;
    }
}

/// Send a GET request to the digital twin provider and return the resulting value.
///
/// # Arguments
/// `provider_uri` - The provider's URI.
/// `entity_id` - The entity id.
async fn send_get_request(provider_uri: &str, entity_id: &str) -> Result<String, Status> {
    info!("Sending a get request to provider URI {provider_uri} for the value of {entity_id}");
    let mut client = DigitalTwinProviderTutorialClient::connect(provider_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(GetRequest { entity_id: entity_id.to_string() });
    let response = client.get(request).await?;

    Ok(response.into_inner().property_value)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The digital twin consumer has started.");

    let settings = consumer_config::load_settings();

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    // Acquire the provider's endpoint for the show notification command
    let show_notification_command_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::hmi::show_notification::ID,
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::INVOKE.to_string()],
        )
        .await
        .unwrap();
    let show_notification_command_provider_uri =
        show_notification_command_provider_endpoint_info.uri;

    // Acquire the provider's endpoint for the ambient air temperature signal
    let mut provider_uri_map = HashMap::new();
    let ambient_air_temperature_property_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::hvac::ambient_air_temperature::ID,
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::GET.to_string()],
        )
        .await
        .unwrap();
    provider_uri_map.insert(
        sdv::hvac::ambient_air_temperature::ID.to_string(),
        ambient_air_temperature_property_provider_endpoint_info.uri,
    );

    // Acquire the provider's endpoint for the is air conditioning active signal
    let is_air_conditioning_active_property_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::hvac::is_air_conditioning_active::ID,
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::GET.to_string()],
        )
        .await
        .unwrap();
    provider_uri_map.insert(
        sdv::hvac::is_air_conditioning_active::ID.to_string(),
        is_air_conditioning_active_property_provider_endpoint_info.uri,
    );

    tokio::select! {
        _ = start_show_notification_repeater(show_notification_command_provider_uri.clone()) => {}
        _ = start_get_signals_repeater(provider_uri_map) => {}
    }

    debug!("The Consumer has completed.");

    Ok(())
}
