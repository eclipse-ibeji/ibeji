// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::agemo::publisher::v1::{ManageTopicRequest, ManageTopicResponse};
use core_protobuf_data_access::agemo::publisher::v1::publisher_callback_server::{PublisherCallback, PublisherCallbackServer};
use core_protobuf_data_access::agemo::pubsub::v1::pub_sub_client::PubSubClient;
use core_protobuf_data_access::agemo::pubsub::v1::{CreateTopicRequest, CreateTopicResponse, DeleteTopicResponse, DeleteTopicRequest};
use core_protobuf_data_access::extensions::managed_subscribe::v1::managed_subscribe_callback_client::ManagedSubscribeCallbackClient;
use core_protobuf_data_access::extensions::managed_subscribe::v1::managed_subscribe_server::{ManagedSubscribe, ManagedSubscribeServer};
use core_protobuf_data_access::extensions::managed_subscribe::v1::{
    SubscriptionInfoRequest, SubscriptionInfoResponse, CallbackPayload, TopicManagementRequest, SubscriptionInfo,
};

use log::{debug, error, info};
use parking_lot::RwLock;
use serde_derive::Deserialize;
use tonic::transport::server::Router;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::{Display, EnumString};
use tonic::{Request, Response, Status};

use crate::extension::GrpcExtensionService;
use crate::extension_config::load_settings;
use crate::managed_subscribe::managed_subscribe_store::{CallbackInfo, SubscriptionStore, TopicInfo};

pub const AGEMO_ENDPOINT: &str = "http://0.0.0.0:50051";
pub const CONFIG_FILENAME: &str = "invehicle_digital_twin_settings";
pub const MS_PROTOCOL: &str = "grpc";

/// Actions that are returned from the Pub Sub Service.
#[derive(Clone, EnumString, Display, Debug, PartialEq)]
pub enum TopicAction {
    /// Enum for the intitial state of a topic.
    #[strum(serialize = "INIT")]
    Init,
    /// Enum correlating to a START action from the Pub Sub Service.
    #[strum(serialize = "START")]
    Start,
    /// Enum correlating to a STOP action from the Pub Sub Service.
    #[strum(serialize = "STOP")]
    Stop,
    /// Enum correlating to a DELETE action from the Pub Sub Service.
    #[strum(serialize = "DELETE")]
    Delete,
}

#[derive(Debug, Deserialize)]
pub struct ConfigSettings {
    pub invehicle_digital_twin_authority: String,
    pub chariott_uri: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ManagedSubscribeExt {
    pub managed_subscribe_uri: String,
    pub extension_uri: String,
    pub extension_protocol: String,
    pub subscription_store: Arc<RwLock<SubscriptionStore>>,
}

impl ManagedSubscribeExt {
    /// Creates a new managed subscribe extension object.
    ///
    /// # Arguments
    /// * `extension_uri` - The uri where the extension service will be hosted.
    /// * `subscription_store` - Handle to the shared subscription store for the extension.
    pub fn new(subscription_store: Arc<RwLock<SubscriptionStore>>) -> Self {
        let config = load_settings::<ConfigSettings>(CONFIG_FILENAME);
        let endpoint = config.invehicle_digital_twin_authority;
        let extension_uri = format!("http://{endpoint}");

        ManagedSubscribeExt {
            managed_subscribe_uri: AGEMO_ENDPOINT.to_string(),
            extension_uri: extension_uri.to_string(),
            extension_protocol: MS_PROTOCOL.to_string(),
            subscription_store,
        }
    }

    /// Calls the external managed subscription service to create a new topic.
    ///
    /// # Arguments
    /// * `entity_id` - The entity id to create the new topic for.
    async fn create_managed_topic(
        &self,
        entity_id: &str,
    ) -> Result<Response<CreateTopicResponse>, Status> {
        // Connect to managed subscribe service.
        let mut ms_client = PubSubClient::connect(self.managed_subscribe_uri.to_string())
            .await
            .map_err(|e| {
                error!("Error connecting to pub sub client: {e:?}");
                Status::from_error(Box::new(e))
            })?;
    
        // Construct request.
        let request = Request::new(CreateTopicRequest {
            publisher_id: entity_id.to_string(),
            management_callback: self.extension_uri.clone(),
            management_protocol: self.extension_protocol.clone(),
        });
    
        // Call managed subscribe service.
        ms_client.create_topic(request).await
    }

    /// Calls the external managed subscription service to delete a managed topic.
    ///
    /// # Arguments
    /// * `topic` - The topic to delete.
    async fn delete_managed_topic(
        &self,
        topic: &str,
    ) -> Result<Response<DeleteTopicResponse>, Status> {
        // Connect to managed subscribe service.
        let mut ms_client = PubSubClient::connect(self.managed_subscribe_uri.to_string())
            .await
            .map_err(|e| {
                error!("Error connecting to pub sub client: {e:?}");
                Status::from_error(Box::new(e))
            })?;
        
        // Construct request.
        let request = Request::new(DeleteTopicRequest {
            topic: topic.to_string()
        });

        // Call managed subscribe service.
        ms_client.delete_topic(request).await
    }
}

impl GrpcExtensionService for ManagedSubscribeExt {
    // fn get_services(&self) -> tonic::transport::server::Routes {
    //     let managed_subscribe_service = ManagedSubscribeServer::new(self.clone());
    //     let managed_subscribe_callback_service = PublisherCallbackServer::new(self.clone());
    // }
    fn add_services<L>(&self, builder: Router<L>) -> Router<L> {
        let ext_builder = builder;
        let managed_subscribe_service = ManagedSubscribeServer::new(self.clone());
        let managed_subscribe_callback_service = PublisherCallbackServer::new(self.clone());

        ext_builder
            .add_service(managed_subscribe_service)
            .add_service(managed_subscribe_callback_service)
    }
}

#[tonic::async_trait]
impl ManagedSubscribe for ManagedSubscribeExt {
    /// Get the subscription information for a specific entity id.
    ///
    /// # Arguments
    /// * `request` - Contains entity id and any relevant constraint requests.
    async fn get_subscription_info(
        &self,
        request: Request<SubscriptionInfoRequest>,
    ) -> Result<Response<SubscriptionInfoResponse>, Status> {
        let inner = request.into_inner();
        let entity_id = inner.entity_id;
        let constraints = inner.constraints;

        info!("Received a get_subscription_info request for entity id {entity_id}");

        // Check if store contains entity.
        {
            let contains_entity = self.subscription_store.read().contains_entity(&entity_id);

            if !contains_entity {
                return Err(Status::not_found("Unable to get dynamic subscription for {entity_id}"));
            };
        } 

        // See if there is a topic that matches requested constraints

        // Get managed subscribe topic information.
        let created_topic = self.create_managed_topic(&entity_id).await?.into_inner();
    
        let generated_topic = created_topic.generated_topic;

        // Save topic information.
        let topic_info = TopicInfo {
            uri: created_topic.broker_uri,
            protocol: created_topic.broker_protocol,
            constraints
        };

        // Add topic to store.
        {
            self.subscription_store.write().add_topic(&entity_id, &generated_topic, topic_info.clone());
        }

        // Respond with subscription information.
        let response = SubscriptionInfoResponse {
            protocol: topic_info.protocol,
            uri: topic_info.uri,
            context: generated_topic,
        };

        debug!("Responded to the get_subscription_info request.");
    
        Ok(Response::new(response))
    }
}

#[tonic::async_trait]
impl PublisherCallback for ManagedSubscribeExt {
    /// Callback for managing a topic based on subscriptions.
    ///
    /// # Arguments
    /// * `request` - Topic and action to take on the topic.
    async fn manage_topic_callback(
        &self,
        request: Request<ManageTopicRequest>,
    ) -> Result<Response<ManageTopicResponse>, Status> {
        info!("Manage_topic_callback called");
        let inner = request.into_inner();
        let topic = inner.topic;
        let topic_action = TopicAction::from_str(inner.action.as_str())
            .map_err(|e| Status::not_found(format!("no valid action was found: {e}")))?;

        let callback_info: CallbackInfo;
        let entity_id: String;
        let topic_info: TopicInfo;

        // Flag to delete topic and remove from store if appropriate.
        let mut delete_topic = false;

        // Get entity information from topic.
        {
            let store = self.subscription_store.read();

            // Get associated entity id with the topic name.
            entity_id = match store.get_entity_id(&topic) {
                Some(id) => id.clone(),
                None => {
                    return Err(Status::not_found(format!("No mapping found for {topic}.")));
                }
            };

            // Get the associated provider and entity metadata using entity id.
            let entity_metadata = match store.get_entity_metadata(&entity_id) {
                Some(metadata) => metadata,
                None => {
                    return Err(Status::not_found(format!("No mapping found for {entity_id}.")));
                }
            };

            // Pull out necessary topic information.
            callback_info = entity_metadata.callback.clone();
            topic_info = entity_metadata.topics.get(&topic).unwrap().clone();
        }

        // Construct management request.
        let management_request = match topic_action {
            TopicAction::Start => {
                let action = String::from("PUBLISH");
                let payload = CallbackPayload {
                    entity_id,
                    topic: topic.clone(),
                    constraints: topic_info.constraints,
                    subscription_info: Some(SubscriptionInfo {
                        protocol: topic_info.protocol,
                        uri: topic_info.uri,
                    }),
                };
                TopicManagementRequest {
                    action,
                    payload: Some(payload),
                }
            }
            TopicAction::Stop => {
                let action = String::from("STOP_PUBLISH");
                let payload = CallbackPayload {
                    entity_id,
                    topic: topic.clone(),
                    constraints: topic_info.constraints,
                    subscription_info: None,
                };

                delete_topic = true;

                TopicManagementRequest {
                    action,
                    payload: Some(payload),
                }
            }
            _ => {
                info!("action is: {topic_action}");
                return Ok(Response::new(ManageTopicResponse {}));
            }
        };

        // Send management request to provider.
        let mut provider_cb_client = ManagedSubscribeCallbackClient::connect(callback_info.uri).await.map_err(|e| {
            error!("Error connecting to provider cb client: {e:?}");
            Status::from_error(Box::new(e))
        })?;
    
        let _res = provider_cb_client.topic_management_cb(management_request).await.map_err(|e| {
            error!("Error calling to provider cb client: {e:?}");
            Status::from_error(Box::new(e))
        })?;

        if delete_topic {
            // Delete topic from managed subscribe service.
            self.delete_managed_topic(&topic).await?;

            // Remove topic from store.
            {
                self.subscription_store.write().remove_topic(&topic);
            }
        }

        Ok(Response::new(ManageTopicResponse {}))
    }
}
