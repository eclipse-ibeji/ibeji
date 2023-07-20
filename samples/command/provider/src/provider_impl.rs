// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use log::{debug, info, warn};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::RespondRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProvider;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
    SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use serde_derive::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

/// The reponse payload is empty.
#[derive(Debug, Serialize, Deserialize)]
struct ResponsePayload {}

#[derive(Debug, Default)]
pub struct ProviderImpl {}

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
        warn!("Got a subscribe request: {request:?}");

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
        warn!("Got a get request: {request:?}");

        Err(Status::unimplemented("get has not been implemented"))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        warn!("Got a set request: {request:?}");

        Err(Status::unimplemented("set has not been implemented"))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let InvokeRequest { entity_id, response_id, consumer_uri, payload } = request.into_inner();

        let request_payload_json: serde_json::Value = serde_json::from_str(&payload)
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        let notification_json =
            request_payload_json.get(sdv::hmi::show_notification::request::NAME).unwrap();

        let notification: sdv::hmi::show_notification::request::TYPE =
            serde_json::from_value(notification_json.clone()).unwrap();

        debug!(
            "Received an invoke request from for entity id {entity_id} with payload'{payload}' from consumer URI {consumer_uri}"
        );

        info!("Notification: '{notification}'");

        tokio::spawn(async move {
            let mut client = DigitalTwinConsumerClient::connect(consumer_uri.clone())
                .await
                .map_err(|error| Status::internal(error.to_string()))?;

            let response_payload = ResponsePayload {};
            let response_payload_json = serde_json::to_string(&response_payload).unwrap();

            let respond_request = tonic::Request::new(RespondRequest {
                entity_id: sdv::hmi::show_notification::response::ID.to_string(),
                response_id,
                payload: response_payload_json,
            });

            let response_future = client.respond(respond_request).await;

            debug!(
                "Sent an invoke response to consumer URI {} for entity id {}",
                &consumer_uri, &entity_id
            );

            response_future
        });

        let response = InvokeResponse {};

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod provider_impl_tests {
    use super::*;
    use digital_twin_model::Metadata;
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    struct ShowNotificationRequestPayload {
        #[serde(rename = "Notification")]
        notification: sdv::hmi::show_notification::request::TYPE,
        #[serde(rename = "$metadata")]
        metadata: Metadata,
    }

    #[tokio::test]
    async fn invoke_test() {
        let provider_impl = ProviderImpl {};

        let entity_id = String::from("one-id");
        let consumer_uri = String::from("bogus uri");

        let response_id = Uuid::new_v4().to_string();

        let request_payload: ShowNotificationRequestPayload = ShowNotificationRequestPayload {
            notification: "The show-notification request.".to_string(),
            metadata: Metadata { model: sdv::hmi::show_notification::request::ID.to_string() },
        };
        let request_payload_json = serde_json::to_string(&request_payload).unwrap();

        let request = tonic::Request::new(InvokeRequest {
            entity_id,
            consumer_uri,
            response_id,
            payload: request_payload_json,
        });
        let result = provider_impl.invoke(request).await;
        assert!(result.is_ok());

        // Note: this test does not check that the response has successfully been sent.
    }
}
