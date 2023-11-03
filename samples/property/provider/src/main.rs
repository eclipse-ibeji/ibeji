// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use env_logger::{Builder, Target};
use log::{debug, info, warn, LevelFilter};
use paho_mqtt as mqtt;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::provider_config;
use samples_common::utils::{
    get_uri, retrieve_invehicle_digital_twin_uri, retry_async_based_on_status,
};
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{
    EndpointInfo, EntityAccessInfo, RegisterRequest,
};
use serde_derive::{Deserialize, Serialize};
use tokio::signal;
use tokio::time::{sleep, Duration};
use tonic::Status;

const MQTT_CLIENT_ID: &str = "property-subscriber";

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    #[serde(rename = "AmbientAirTemperature")]
    ambient_air_temperature: sdv::hvac::ambient_air_temperature::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Register the ambient air temperature property's endpoint.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `broker_uri` - The broker's URI.
/// * `topic` - The topic.
async fn register_ambient_air_temperature(
    invehicle_digital_twin_uri: &str,
    broker_uri: &str,
    topic: &str,
) -> Result<(), Status> {
    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::MQTT.to_string(),
        operations: vec![digital_twin_operation::SUBSCRIBE.to_string()],
        uri: broker_uri.to_string(),
        context: topic.to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: sdv::hvac::ambient_air_temperature::NAME.to_string(),
        id: sdv::hvac::ambient_air_temperature::ID.to_string(),
        description: sdv::hvac::ambient_air_temperature::DESCRIPTION.to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    let mut client = InvehicleDigitalTwinClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;

    Ok(())
}

/// Create the JSON for the ambient air temperature property.
///
/// # Arguments
/// * `ambient_air_temperature` - The ambient air temperature value.
fn create_property_json(ambient_air_temperature: i32) -> String {
    let metadata = Metadata { model: sdv::hvac::ambient_air_temperature::ID.to_string() };

    let property: Property = Property { ambient_air_temperature, metadata };

    serde_json::to_string(&property).unwrap()
}

/// Start the ambient air temperature data stream.
///
/// # Arguments
/// `broker_uri` - The host.
/// `topic` - The topic.
fn start_ambient_air_temperature_data_stream(broker_uri: String, topic: String) {
    debug!("Starting the Provider's ambient air temperature data stream.");
    tokio::spawn(async move {
        let mut temperature: i32 = 75;
        let mut is_temperature_increasing: bool = true;
        loop {
            let content = create_property_json(temperature);

            info!(
                "Sending a publish request for {} with value {temperature}",
                sdv::hvac::ambient_air_temperature::ID
            );
            if let Err(err) = publish_message(&broker_uri, &topic, &content) {
                warn!("Publish request failed due to '{err:?}'");
                break;
            }

            debug!("Completed the publish request");

            // Calculate the new temperature.
            // It bounces back and forth between 65 and 85 degrees.
            if is_temperature_increasing {
                if temperature == 85 {
                    is_temperature_increasing = false;
                    temperature -= 1;
                } else {
                    temperature += 1;
                }
            } else if temperature == 65 {
                is_temperature_increasing = true;
                temperature += 1;
            } else {
                temperature -= 1;
            }

            sleep(Duration::from_secs(5)).await;
        }
    });
}

/// Publish a message to a MQTT broker located.
///
/// # Arguments
/// `broker_uri` - The MQTT broker's URI.
/// `topic` - The topic to publish to.
/// `content` - The message to publish.
fn publish_message(broker_uri: &str, topic: &str, content: &str) -> Result<(), String> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker_uri)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();

    let client = mqtt::Client::new(create_opts)
        .map_err(|err| format!("Failed to create the client due to '{err:?}'"))?;

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(30))
        .clean_session(true)
        .finalize();

    let _connect_response =
        client.connect(conn_opts).map_err(|err| format!("Failed to connect due to '{err:?}"));

    let msg = mqtt::Message::new(topic, content, mqtt::types::QOS_1);
    if let Err(err) = client.publish(msg) {
        return Err(format!("Failed to publish message due to '{err:?}"));
    }

    if let Err(err) = client.disconnect(None) {
        warn!("Failed to disconnect from topic '{topic}' on broker {broker_uri} due to {err:?}");
    }

    Ok(())
}

/// Convert a DTMI to a MQTT topic name.
/// The conversion will strip off the scheme (i.e. the "dtmi:" prefix)
/// and replace all seprators (':' and ';') with a slash.
///
/// # Arguments
/// `dtmi` - The DTMI.
fn convert_dtmi_to_topic(dtmi: &str) -> Result<String, String> {
    let parts: Vec<&str> = dtmi.split(&[':', ';']).collect();
    if parts[0] != "dtmi" {
        return Err("Invalid dtmi".to_string());
    }
    let mut topic: String = "".to_string();
    // We will start at index 1 to skip the scheme.
    for i in 1..parts.len() {
        topic.push_str(parts[i]);
        if i != (parts.len() - 1) {
            topic.push('/');
        }
    }

    Ok(topic)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = provider_config::load_settings();

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    let broker_uri = format!("tcp://{provider_authority}"); // Devskim: ignore DS137138
    debug!("The Broker URI is {}", &broker_uri);

    let topic = convert_dtmi_to_topic(sdv::hvac::ambient_air_temperature::ID)?;
    debug!("Topic is '{topic}'");

    debug!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_ambient_air_temperature(&invehicle_digital_twin_uri, &broker_uri, &topic)
    })
    .await?;

    let stream_uri = get_uri(&broker_uri)?;

    start_ambient_air_temperature_data_stream(stream_uri, topic);

    signal::ctrl_c().await.expect("Failed to listen for control-c event");

    info!("The Provider has completed.");

    Ok(())
}
