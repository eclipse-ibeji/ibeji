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
use tokio::time::Duration;
use tonic::Status;

const MQTT_CLIENT_ID: &str = "property-consumer";

/// Receive Ambient Air Temperature updates.
///
/// # Arguments
/// * `broker_uri` - The broker URI.
/// * `topic` - The topic.
fn receive_ambient_air_temperature_updates(broker_uri: &str, topic: &str) -> Result<(), String> {
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

    for msg in receiver.iter() {
        if let Some(msg) = msg {
            info!("{}", msg);
        } else if !client.is_connected() {
            if client.reconnect().is_ok() {
                _subscribe_response = client.subscribe(topic, mqtt::types::QOS_1).map_err(|err| {
                    format!("Failed to subscribe to topic {topic} due to '{err:?}'")
                });
            } else {
                break;
            }
        }
    }

    if client.is_connected() {
        debug!("Disconnecting");
        client.unsubscribe(topic).unwrap();
        client.disconnect(None).unwrap();
    }

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

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::hvac::ambient_air_temperature::ID,
        digital_twin_protocol::MQTT,
        &[digital_twin_operation::SUBSCRIBE.to_string()],
    )
    .await
    .unwrap();
    let broker_uri = provider_endpoint_info.uri;
    info!("The Broker URI for the AmbientAirTemperature property's provider is {broker_uri}");
    let topic = provider_endpoint_info.context;
    info!("The Topic for the AmbientAirTemperature property's provider is {topic})");

    let _receive_result = receive_ambient_air_temperature_updates(&broker_uri, &topic)
        .map_err(|err| Status::internal(format!("{err:?}")));

    info!("The Consumer has completed.");

    Ok(())
}
