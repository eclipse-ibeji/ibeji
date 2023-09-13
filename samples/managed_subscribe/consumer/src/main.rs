// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use paho_mqtt as mqtt;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::consumer_config;
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::extensions::managed_subscribe::v1::{SubscriptionInfoRequest, Constraint, SubscriptionInfoResponse};
use samples_protobuf_data_access::extensions::managed_subscribe::v1::managed_subscribe_client::ManagedSubscribeClient;
use tokio::signal;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::Duration;
use tonic::{Status, Request};

const MQTT_CLIENT_ID: &str = "managed-subscribe-consumer";

/// Get subscription information from managed subscribe endpoint.
///
/// # Arguments
/// * `managed_subscribe_uri` - The managed subscribe URI.
/// * `constraints` - Constraints for the managed topic.
async fn get_ambient_air_temperature_subscription_info(
    managed_subscribe_uri: &str,
    constraints: Vec<Constraint>,
) -> Result<SubscriptionInfoResponse, Status> {
    // Create gRPC client.
    let mut client = ManagedSubscribeClient::connect(managed_subscribe_uri.to_string())
        .await
        .map_err(|err| Status::from_error(err.into()))?;

    let request = Request::new(SubscriptionInfoRequest {
        entity_id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        constraints,
    });

    let response = client.get_subscription_info(request).await?;
    let subscription_info = response.into_inner().clone();

    Ok(subscription_info)
}

/// Receive Ambient Air Temperature updates.
///
/// # Arguments
/// * `broker_uri` - The broker URI.
/// * `topic` - The topic.
fn receive_ambient_air_temperature_updates(broker_uri: &str, topic: &str) -> Result<Sender<bool>, String> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker_uri)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();

    let client = mqtt::Client::new(create_opts)
        .map_err(|err| format!("Failed to create the client due to '{err:?}'"))?;

    let receiver = client.start_consuming();

    // Last Will and Testament
    let lwt =
        mqtt::MessageBuilder::new().topic("test").payload("Receiver lost connection").finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(30))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    let _connect_response =
        client.connect(conn_opts).map_err(|err| format!("Failed to connect due to '{err:?}"));

    let mut _subscribe_response = client
        .subscribe(topic, mqtt::types::QOS_1)
        .map_err(|err| format!("Failed to subscribe to topic {topic} due to '{err:?}'"));

    let (shutdown_sender, mut shutdown_receiver) = mpsc::channel::<bool>(1);

    // Copy topic for separate thread.
    let topic_string = topic.to_string();

    tokio::spawn(async move {
        for msg in receiver.iter().take_while(|_| {
            match shutdown_receiver.try_recv() {
                Err(TryRecvError::Disconnected) => false,
                _ => true,
            }
        }) {
            if let Some(msg) = msg {
                info!("{}", msg);
            } else if !client.is_connected() {
                if client.reconnect().is_ok() {
                    _subscribe_response = client.subscribe(topic_string.as_str(), mqtt::types::QOS_1).map_err(|err| {
                        format!("Failed to subscribe to topic {topic_string} due to '{err:?}'")
                    });
                } else {
                    break;
                }
            }
        }
    
        if client.is_connected() {
            debug!("Disconnecting");
            client.unsubscribe(topic_string.as_str()).unwrap();
            client.disconnect(None).unwrap();
        }
    });

    Ok(shutdown_sender)
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

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::MANAGEDSUBSCRIBE.to_string()],
    )
    .await
    .unwrap();

    let managed_subscribe_uri = provider_endpoint_info.uri;
    info!("The Managed Subscribe URI for the AmbientAirTemperature property's provider is {managed_subscribe_uri}");

    // Create constraint for the managed subscribe call.
    let frequency_ms: u64 = 10000;

    let frequency_constraint = Constraint {
        r#type: String::from("frequency"),
        value: frequency_ms.to_string(),
    };

    // Get the subscription information for a managed topic with constraints.
    let subscription_info = get_ambient_air_temperature_subscription_info(
        &managed_subscribe_uri,
        vec![frequency_constraint],
    ).await?;

    // Deconstruct subscription information.
    let broker_uri = subscription_info.uri;
    let topic = subscription_info.context;
    info!("The broker URI for the AmbientAirTemperature property's provider is {broker_uri}");

    // Subscribe to topic.
    let shutdown_sender = receive_ambient_air_temperature_updates(&broker_uri, &topic)
        .map_err(|err| Status::internal(format!("{err:?}")))?;

    signal::ctrl_c().await?;

    drop(shutdown_sender);

    info!("The Consumer has completed. Shutting down...");

    Ok(())
}
