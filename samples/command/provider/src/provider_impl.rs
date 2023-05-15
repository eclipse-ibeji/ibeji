// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::{info, warn};
use parking_lot::Mutex;
use samples_proto::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_client::DigitalTwinConsumerClient;
use samples_proto::sample_grpc::v1::digital_twin_consumer::RespondRequest;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProvider;
use samples_proto::sample_grpc::v1::digital_twin_provider::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
    SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub type SubscriptionMap = HashMap<String, HashSet<String>>;

#[derive(Debug, Default)]
pub struct ProviderImpl {
    pub subscription_map: Arc<Mutex<SubscriptionMap>>,
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
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let response_id: String = request_inner.response_id.clone();
        let consumer_uri: String = request_inner.consumer_uri;
        let payload: String = request_inner.payload;

        info!(
            "Received an invoke request from for entity id {entity_id} with payload '{payload}' from consumer URI {consumer_uri}"
        );

        info!("Notification: '{payload}'");

        tokio::spawn(async move {
            let mut client = DigitalTwinConsumerClient::connect(consumer_uri.clone())
                .await
                .map_err(|error| Status::internal(error.to_string()))?;

            let respond_request = tonic::Request::new(RespondRequest {
                entity_id: entity_id.clone(),
                response_id,
                payload,
            });

            let response_future = client.respond(respond_request).await;

            info!(
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
    use uuid::Uuid;

    #[tokio::test]
    async fn invoke_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let provider_impl = ProviderImpl { subscription_map };

        let entity_id = String::from("one-id");
        let consumer_uri = String::from("bogus uri");

        let response_id = Uuid::new_v4().to_string();
        let payload = String::from("some-payload");

        let request =
            tonic::Request::new(InvokeRequest { entity_id, consumer_uri, response_id, payload });
        let result = provider_impl.invoke(request).await;
        assert!(result.is_ok());

        // Note: this test does not check that the response has successfully been sent.
    }
}
