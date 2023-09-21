// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::agemo::publisher::v1::publisher_callback_server::{
    PublisherCallback, PublisherCallbackServer,
};
use core_protobuf_data_access::agemo::publisher::v1::{ManageTopicRequest, ManageTopicResponse};
use core_protobuf_data_access::agemo::pubsub::v1::pub_sub_client::PubSubClient;
use core_protobuf_data_access::agemo::pubsub::v1::{
    CreateTopicRequest, CreateTopicResponse, DeleteTopicRequest, DeleteTopicResponse,
};
use core_protobuf_data_access::module::managed_subscribe::v1::managed_subscribe_callback_client::ManagedSubscribeCallbackClient;
use core_protobuf_data_access::module::managed_subscribe::v1::managed_subscribe_server::{
    ManagedSubscribe, ManagedSubscribeServer,
};
use core_protobuf_data_access::module::managed_subscribe::v1::{
    CallbackPayload, SubscriptionInfo, SubscriptionInfoRequest, SubscriptionInfoResponse,
    TopicManagementRequest,
};

use common::grpc_module::GrpcModule;
use common::utils::{execute_with_retry, load_settings};
use log::{debug, error, info};
use parking_lot::RwLock;
use serde_derive::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::{Display, EnumString};
use tonic::transport::server::RoutesBuilder;
use tonic::{Request, Response, Status};

use crate::managed_subscribe_store::{CallbackInfo, ManagedSubscribeStore, TopicInfo};

use super::managed_subscribe_interceptor::ManagedSubscribeInterceptor;

const CONFIG_FILENAME: &str = "managed_subscribe_settings";
const EXTENSION_PROTOCOL: &str = "grpc";

// Managed Subscribe action constants.
const PUBLISH_ACTION: &str = "PUBLISH";
const STOP_PUBLISH_ACTION: &str = "STOP_PUBLISH";

/// Actions that are returned from the Pub Sub Service.
#[derive(Clone, EnumString, Eq, Display, Debug, PartialEq)]
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
    pub base_authority: String,
    pub managed_subscribe_uri: String,
    pub chariott_uri: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ManagedSubscribeExt {
    pub managed_subscribe_uri: String,
    pub extension_uri: String,
    pub extension_protocol: String,
    pub extension_store: Arc<RwLock<ManagedSubscribeStore>>,
}

impl Default for ManagedSubscribeExt {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagedSubscribeExt {
    /// Creates a new managed subscribe extension object.
    pub fn new() -> Self {
        // Get extension information from the configuration settings.
        let config = load_settings::<ConfigSettings>(CONFIG_FILENAME);
        let endpoint = config.base_authority;
        let extension_uri = format!("http://{endpoint}"); // Devskim: ignore DS137138

        let extension_store = Arc::new(RwLock::new(ManagedSubscribeStore::new()));

        ManagedSubscribeExt {
            managed_subscribe_uri: config.managed_subscribe_uri,
            extension_uri,
            extension_protocol: EXTENSION_PROTOCOL.to_string(),
            extension_store,
        }
    }

    /// Creates a new managed subscribe interceptor that shares data with the current instance of
    /// this extension.
    pub fn create_interceptor(&self) -> ManagedSubscribeInterceptor {
        ManagedSubscribeInterceptor::new(&self.extension_uri, self.extension_store.clone())
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
        let mut ms_client =
            PubSubClient::connect(self.managed_subscribe_uri.to_string()).await.map_err(|e| {
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
        let mut ms_client =
            PubSubClient::connect(self.managed_subscribe_uri.to_string()).await.map_err(|e| {
                error!("Error connecting to pub sub client: {e:?}");
                Status::from_error(Box::new(e))
            })?;

        // Construct request.
        let request = Request::new(DeleteTopicRequest { topic: topic.to_string() });

        // Call managed subscribe service.
        ms_client.delete_topic(request).await
    }
}

impl GrpcModule for ManagedSubscribeExt {
    /// Adds the gRPC services for this extension to the server builder.
    ///
    /// # Arguments
    /// * `builder` - A tonic::RoutesBuilder that contains the grpc services to build.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder) {
        // Create the gRPC services.
        let managed_subscribe_service = ManagedSubscribeServer::new(self.clone());
        let managed_subscribe_callback_service = PublisherCallbackServer::new(self.clone());

        builder
            .add_service(managed_subscribe_service)
            .add_service(managed_subscribe_callback_service);
    }
}

/// Calls a provider's callback endpoint with a management request.
///
/// # Arguments
/// * `provider_cb_uri` - The provider's callback uri.
/// * `management_request` - The topic management request to send.
async fn call_provider_management_cb(
    provider_cb_uri: &str,
    management_request: TopicManagementRequest,
) -> Result<(), Status> {
    let mut provider_cb_client =
        ManagedSubscribeCallbackClient::connect(provider_cb_uri.to_string()).await.map_err(
            |e| {
                error!("Error connecting to provider cb client: {e:?}");
                Status::from_error(Box::new(e))
            },
        )?;

    let _res = provider_cb_client.topic_management_cb(management_request).await.map_err(|e| {
        error!("Error calling to provider cb client: {e:?}");
        Status::from_error(Box::new(e))
    })?;

    Ok(())
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
            let contains_entity = self.extension_store.read().contains_entity(&entity_id);

            if !contains_entity {
                return Err(Status::not_found(
                    "Unable to get dynamic subscription for {entity_id}",
                ));
            };
        }

        // Get managed subscribe topic information.
        let created_topic = execute_with_retry(
            30,
            tokio::time::Duration::from_secs(1),
            || self.create_managed_topic(&entity_id),
            Some(format!("create_managed_topic({entity_id})")),
        )
        .await?
        .into_inner();

        let generated_topic = created_topic.generated_topic;

        // Save topic information.
        let topic_info = TopicInfo {
            uri: created_topic.broker_uri,
            protocol: created_topic.broker_protocol,
            constraints,
        };

        // Add topic to store.
        {
            self.extension_store.write().add_topic(
                &entity_id,
                &generated_topic,
                topic_info.clone(),
            );
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
            let store = self.extension_store.read();

            // Get associated entity id with the topic name.
            entity_id = store
                .get_entity_id(&topic)
                .ok_or_else(|| Status::not_found(format!("No mapping found for {topic}.")))?
                .to_string();

            // Get the associated provider and entity metadata using entity id.
            let entity_metadata = store
                .get_entity_metadata(&entity_id)
                .ok_or_else(|| Status::not_found(format!("No mapping found for {entity_id}.")))?;

            // Pull out necessary topic information.
            callback_info = entity_metadata.callback.clone();
            topic_info = entity_metadata.topics.get(&topic).unwrap().clone();
        }

        // Construct management request.
        let management_request = match topic_action {
            TopicAction::Start => {
                let action = String::from(PUBLISH_ACTION);
                let payload = CallbackPayload {
                    entity_id,
                    topic: topic.clone(),
                    constraints: topic_info.constraints,
                    subscription_info: Some(SubscriptionInfo {
                        protocol: topic_info.protocol,
                        uri: topic_info.uri,
                    }),
                };

                TopicManagementRequest { action, payload: Some(payload) }
            }
            TopicAction::Stop => {
                let action = String::from(STOP_PUBLISH_ACTION);
                let payload = CallbackPayload {
                    entity_id,
                    topic: topic.clone(),
                    constraints: topic_info.constraints,
                    subscription_info: None,
                };

                delete_topic = true;

                TopicManagementRequest { action, payload: Some(payload) }
            }
            _ => {
                info!("action is: {topic_action}");
                return Ok(Response::new(ManageTopicResponse {}));
            }
        };

        // Send management request to provider.
        execute_with_retry(
            30,
            tokio::time::Duration::from_secs(1),
            || call_provider_management_cb(&callback_info.uri, management_request.clone()),
            Some("call_provider_management_cb".to_string()),
        )
        .await?;

        if delete_topic {
            // Delete topic from managed subscribe service.
            execute_with_retry(
                30,
                tokio::time::Duration::from_secs(1),
                || self.delete_managed_topic(&topic),
                Some(format!("delete_managed_topic{topic}")),
            )
            .await?;

            // Remove topic from store.
            {
                self.extension_store.write().remove_topic(&topic);
            }
        }

        Ok(Response::new(ManageTopicResponse {}))
    }
}
