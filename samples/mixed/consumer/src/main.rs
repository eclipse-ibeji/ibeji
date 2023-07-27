// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod consumer_impl;

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_common::consumer_config;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
    InvokeRequest, SetRequest, SubscribeRequest,
};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tonic::{Status, transport::Server};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct IsAirConditioningActiveProperty {
    #[serde(rename = "IsAirConditioningActive")]
    is_air_conditioning_active: sdv::hvac::is_air_conditioning_active::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShowNotificationRequestPayload {
    #[serde(rename = "Notification")]
    notification: sdv::hmi::show_notification::request::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Start the show-notification repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
/// `consumer_uri` - The consumer_uri.
fn start_show_notification_repeater(provider_uri: String, consumer_uri: String) {
    debug!("Starting the Consumer's show-notification repeater.");

    tokio::spawn(async move {
        let metadata = Metadata { model: sdv::hmi::show_notification::request::ID.to_string() };

        let request_payload: ShowNotificationRequestPayload = ShowNotificationRequestPayload {
            notification: "The show-notification request.".to_string(),
            metadata,
        };

        let request_payload_json = serde_json::to_string(&request_payload).unwrap();

        loop {
            info!("Sending an invoke request on entity {} with payload '{} to provider URI {provider_uri}",
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

            debug!("Completed the invoke request");

            sleep(Duration::from_secs(55)).await;
        }
    });
}

/// Start the activate-air-conditioing repeater.
///
/// # Arguments
/// `provider_uri` - The provider_uri.
fn start_activate_air_conditioning_repeater(provider_uri: String) {
    debug!("Starting the Consumer's activate-air-conditioning repeater.");

    tokio::spawn(async move {
        let mut is_active = true;

        loop {
            info!("Sending a set request for entity id {} to the value '{is_active}' to provider URI {provider_uri}",
                sdv::hvac::is_air_conditioning_active::ID);

            let metadata: Metadata =
                Metadata { model: sdv::hvac::is_air_conditioning_active::ID.to_string() };
            let property: IsAirConditioningActiveProperty =
                IsAirConditioningActiveProperty { is_air_conditioning_active: is_active, metadata };

            let value = serde_json::to_string_pretty(&property).unwrap();

            let client_result = DigitalTwinProviderClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will retry in a moment.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let mut client = client_result.unwrap();

            let request = tonic::Request::new(SetRequest {
                entity_id: sdv::hvac::is_air_conditioning_active::ID.to_string(),
                value,
            });

            let response = client.set(request).await;
            match response {
                Ok(_) => is_active = !is_active,
                Err(status) => warn!("{status:?}"),
            }

            debug!("Completed the set request.");

            sleep(Duration::from_secs(30)).await;
        }
    });
}

/// Send a subscribe request.
///
/// # Arguments
/// `provider_uri` - The provider's URI.
/// `entity_id` - The entity id.
/// `consumer_uri` - The consumer's URI.
async fn send_subscribe_request(
    provider_uri: &str,
    entity_id: &str,
    consumer_uri: &str,
) -> Result<(), Status> {
    info!("Sending a subscribe request for entity id {entity_id} to provider URI {provider_uri}");
    let mut client = DigitalTwinProviderClient::connect(provider_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(SubscribeRequest {
        entity_id: entity_id.to_string(),
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

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    let consumer_authority = settings
        .consumer_authority
        .expect("consumer_authority must be specified in the config file");

    let consumer_uri = format!("http://{consumer_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = consumer_authority.parse()?;
    let consumer_impl = consumer_impl::ConsumerImpl::default();
    let server_future =
        Server::builder().add_service(DigitalTwinConsumerServer::new(consumer_impl)).serve(addr);
    info!("The HTTP server is listening on address '{consumer_authority}'");

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

    let ambient_air_temperature_property_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::hvac::ambient_air_temperature::ID,
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::SUBSCRIBE.to_string()],
        )
        .await
        .unwrap();
    let ambient_air_temperature_property_provider_uri =
        ambient_air_temperature_property_provider_endpoint_info.uri;

    let is_air_conditioning_active_property_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::hvac::is_air_conditioning_active::ID,
            digital_twin_protocol::GRPC,
            &[
                digital_twin_operation::SUBSCRIBE.to_string(),
                digital_twin_operation::SET.to_string(),
            ],
        )
        .await
        .unwrap();
    let is_air_conditioning_active_property_provider_uri =
        is_air_conditioning_active_property_provider_endpoint_info.uri;

    let hybrid_battery_remaining_property_provider_endpoint_info =
        discover_digital_twin_provider_using_ibeji(
            &invehicle_digital_twin_uri,
            sdv::obd::hybrid_battery_remaining::ID,
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::SUBSCRIBE.to_string()],
        )
        .await
        .unwrap();
    let hybrid_battery_remaining_property_provider_uri =
        hybrid_battery_remaining_property_provider_endpoint_info.uri;

    retry_async_based_on_status(30, Duration::from_secs(1), || {
        send_subscribe_request(
            &ambient_air_temperature_property_provider_uri,
            sdv::hvac::ambient_air_temperature::ID,
            &consumer_uri,
        )
    })
    .await?;

    retry_async_based_on_status(30, Duration::from_secs(1), || {
        send_subscribe_request(
            &is_air_conditioning_active_property_provider_uri,
            sdv::hvac::is_air_conditioning_active::ID,
            &consumer_uri,
        )
    })
    .await?;

    retry_async_based_on_status(30, Duration::from_secs(1), || {
        send_subscribe_request(
            &hybrid_battery_remaining_property_provider_uri,
            sdv::obd::hybrid_battery_remaining::ID,
            &consumer_uri,
        )
    })
    .await?;

    start_activate_air_conditioning_repeater(is_air_conditioning_active_property_provider_uri);

    start_show_notification_repeater(
        show_notification_command_provider_uri.clone(),
        consumer_uri.clone(),
    );

    server_future.await?;

    debug!("The Consumer has completed.");

    Ok(())
}
