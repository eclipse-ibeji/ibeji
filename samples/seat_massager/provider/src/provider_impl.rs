// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::{sdv_v1 as sdv, Metadata};
use log::{debug, info, warn};
use parking_lot::{Mutex, MutexGuard};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
    digital_twin_provider_server::DigitalTwinProvider, GetRequest, GetResponse, InvokeRequest,
    InvokeResponse, SetRequest, SetResponse, SubscribeRequest, SubscribeResponse,
    UnsubscribeRequest, UnsubscribeResponse,
};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::PublishRequest;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use std::vec::Vec;
use tonic::{Request, Response, Status};

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    #[serde(rename = "MassageAirbags")]
    massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE,
    #[serde(rename = "$metadata")]
    metadata: Metadata
}

#[derive(Debug, Default)]
pub struct ProviderProperties {
    pub massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE,
}

#[derive(Debug, Default)]
pub struct ProviderImpl {
    pub properties: Arc<Mutex<ProviderProperties>>,
}

impl ProviderImpl {
    fn set_message_airbags(
        properties: Arc<Mutex<ProviderProperties>>,
        value: &str,
    ) -> Result<(), String> {
        let message_airbags_property_json: serde_json::Value = serde_json::from_str(value).unwrap();
        let message_airbags_json = message_airbags_property_json.get(sdv::airbag_seat_massager::massage_airbags::NAME).unwrap();

        let massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE = serde_json::from_value(message_airbags_json.clone()).unwrap();

        info!("Setting message airbags to: {:?}", massage_airbags);

        // This block controls the lifetime of the lock.
        {
            let mut lock: MutexGuard<ProviderProperties> = properties.lock();

            lock.massage_airbags = massage_airbags;
        }

        Ok(())
    }

    fn get_message_airbags(
        properties: Arc<Mutex<ProviderProperties>>
    ) -> Result<String, String> {
        let metadata = Metadata {
            model: sdv::airbag_seat_massager::massage_airbags::ID.to_string(),
        };

        let mut property: Property = Property {
            massage_airbags: Vec::new(),
            metadata,
        };

        // This block controls the lifetime of the lock.
        {
            let lock: MutexGuard<ProviderProperties> = properties.lock();

            property.massage_airbags = lock.massage_airbags.clone();
        }

        let json_str = serde_json::to_string(&property).unwrap();

        Ok(json_str)
    }
}


#[tonic::async_trait]
impl DigitalTwinProvider for ProviderImpl {
    /// Subscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Subscribe request.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        warn!("Got subscribe request: {request:?}");

        Err(Status::unimplemented("subscribe has not been implemented"))
    }

    /// Unsubscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Unsubscribe request.
    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        warn!("Got an unsubscribe request: {request:?}");

        Err(Status::unimplemented("unsubscribe has not been implemented"))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let consumer_uri: String = request_inner.consumer_uri.clone();

        info!("Received a get request for entity id {entity_id}");

        let properties: Arc<Mutex<ProviderProperties>> = self.properties.clone();

        tokio::spawn(async move {
            if entity_id == sdv::airbag_seat_massager::massage_airbags::ID {
                let result = ProviderImpl::get_message_airbags(properties.clone());
                if result.is_err() {
                    warn!("Failed to get {} due to: {}", entity_id, result.unwrap_err());
                    return;
                }
                let client_result = DigitalTwinConsumerClient::connect(consumer_uri).await;
                if client_result.is_err() {
                    warn!("Unable to connect due to {}", client_result.unwrap_err());
                    return;
                }
                let mut client = client_result.unwrap();
    
                let publish_request = tonic::Request::new(PublishRequest {
                    entity_id,
                    value: result.unwrap(),
                });
                let response = client.publish(publish_request).await;
                if let Err(status) = response {
                    warn!("Publish failed: {status:?}");
                    return;
                }
        
            } else {
                warn!("The entity id {entity_id} is not recognized.");
                return;
            }
        });

        let response = GetResponse {};

        debug!("Completed the get request.");

        Ok(Response::new(response))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let value: String = request_inner.value;

        info!("Received a set request for entity id {entity_id}");

        let properties: Arc<Mutex<ProviderProperties>> = self.properties.clone();

        tokio::spawn(async move {
            if entity_id == sdv::airbag_seat_massager::massage_airbags::ID {
                let result = ProviderImpl::set_message_airbags(properties.clone(), &value);
                if result.is_err() {
                    warn!("Failed to set {} due to: {}", entity_id, result.unwrap_err());
                }
            } else {
                warn!("Error: The entity id {entity_id} is not recognized.");
            }
        });

        let response = SetResponse {};

        debug!("Completed the set request.");

        Ok(Response::new(response))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        warn!("Got an invoke request: {request:?}");

        Err(Status::unimplemented("invoke has not been implemented"))
    }
}  
