// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use samples_protobuf_data_access::extension::managed_subscribe::v1::managed_subscribe_callback_server::ManagedSubscribeCallback;
use samples_protobuf_data_access::extension::managed_subscribe::v1::{
    CallbackPayload, TopicManagementRequest, TopicManagementResponse,
};

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use log::{debug, info, warn};
use paho_mqtt as mqtt;
use parking_lot::RwLock;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

const MQTT_CLIENT_ID: &str = "managed-subscribe-publisher";

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    #[serde(rename = "AmbientAirTemperature")]
    ambient_air_temperature: sdv::hvac::ambient_air_temperature::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata,
}

/// Actions that are returned from the Pub Sub Service.
#[derive(Clone, EnumString, Eq, Display, Debug, PartialEq)]
pub enum ProviderAction {
    #[strum(serialize = "PUBLISH")]
    Publish,

    #[strum(serialize = "STOP_PUBLISH")]
    StopPublish,
}

#[derive(Debug)]
pub struct TopicInfo {
    topic: String,
    stop_channel: mpsc::Sender<bool>,
}

#[derive(Debug)]
pub struct ProviderImpl {
    pub data_stream: watch::Receiver<i32>,
    pub min_interval_ms: u64,
    entity_map: Arc<RwLock<HashMap<String, Vec<TopicInfo>>>>,
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

impl ProviderImpl {
    /// Initializes provider with entities relevant to itself.
    ///
    /// # Arguments
    /// * `data_stream` - Receiver for data stream for entity.
    /// * `min_interval_ms` - The frequency of the data coming over the data stream.
    pub fn new(data_stream: watch::Receiver<i32>, min_interval_ms: u64) -> Self {
        // Initialize entity map.
        let mut entity_map = HashMap::new();

        // Insert entry for entity id's associated with provider.
        entity_map.insert(sdv::hvac::ambient_air_temperature::ID.to_string(), Vec::new());

        // Create new instance.
        ProviderImpl { data_stream, min_interval_ms, entity_map: Arc::new(RwLock::new(entity_map)) }
    }

    /// Handles the 'PUBLISH' action from the callback.
    ///
    /// # Arguments
    /// `payload` - Payload sent with the 'PUBLISH' action.
    pub fn handle_publish_action(&self, payload: CallbackPayload) {
        // Get payload information.
        let topic = payload.topic;
        let constraints = payload.constraints;
        let min_interval_ms = self.min_interval_ms;

        // This should not be empty.
        let subscription_info = payload.subscription_info.unwrap();

        // Create stop publish channel.
        let (sender, mut reciever) = mpsc::channel(10);

        // Create topic info.
        let topic_info = TopicInfo { topic: topic.clone(), stop_channel: sender };

        // Record new topic in entity map.
        {
            let mut entity_lock = self.entity_map.write();
            let get_result = entity_lock.get_mut(&payload.entity_id);
            get_result.unwrap().push(topic_info);
        }

        let data_stream = self.data_stream.clone();

        // Start thread for new topic.
        tokio::spawn(async move {
            // Get constraints information.
            let mut frequency_ms = min_interval_ms;

            for constraint in constraints {
                if constraint.r#type == *"frequency" {
                    frequency_ms = u64::from_str(&constraint.value).unwrap();
                };
            }

            loop {
                // See if we need to shutdown.
                match reciever.try_recv() {
                    Ok(_) => {
                        info!("Shutdown thread for {topic}.");
                        return;
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {}
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        info!("Shutdown thread for {topic}.");
                        return;
                    }
                };

                // Get data from stream at the current instant.
                let data = *data_stream.borrow();
                let content = create_property_json(data);
                let broker_uri = subscription_info.uri.clone();

                // Publish message to broker.
                info!(
                    "Sending a publish request for {} with value {data}",
                    sdv::hvac::ambient_air_temperature::ID
                );

                if let Err(err) = publish_message(&broker_uri, &topic, &content) {
                    warn!("Publish request failed due to '{err:?}'");
                    break;
                }

                debug!("Completed the publish request");

                // Sleep for requested amount of time.
                sleep(Duration::from_millis(frequency_ms)).await;
            }
        });
    }

    /// Handles the 'STOP_PUBLISH' action from the callback.
    ///
    /// # Arguments
    /// `payload` - Payload sent with the 'STOP_PUBLISH' action.
    pub fn handle_stop_publish_action(&self, payload: CallbackPayload) {
        let topic_info: TopicInfo;

        let mut entity_lock = self.entity_map.write();
        let get_result = entity_lock.get_mut(&payload.entity_id);
        let topics = get_result.unwrap();

        // Check to see if topic exists.
        let index_opt = topics.iter_mut().position(|t| t.topic == payload.topic);
        let index = match index_opt {
            Some(i) => i,
            None => {
                warn!("No topic found matching {}", payload.topic);
                return;
            }
        };

        // Remove topic.
        topic_info = topics.swap_remove(index);

        // Stop publishing to removed topic.
        drop(topic_info.stop_channel);
    }
}

#[tonic::async_trait]
impl ManagedSubscribeCallback for ProviderImpl {
    /// Callback for a provider, will process a provider action.
    ///
    /// # Arguments
    /// * `request` - The request with the action and associated payload.
    async fn topic_management_cb(
        &self,
        request: Request<TopicManagementRequest>,
    ) -> Result<Response<TopicManagementResponse>, Status> {
        let inner = request.into_inner();
        let action = inner.action;
        let payload = inner.payload.unwrap();

        let provider_action = ProviderAction::from_str(&action).unwrap();

        match provider_action {
            ProviderAction::Publish => Self::handle_publish_action(self, payload),
            ProviderAction::StopPublish => Self::handle_stop_publish_action(self, payload),
        }

        Ok(Response::new(TopicManagementResponse {}))
    }
}
