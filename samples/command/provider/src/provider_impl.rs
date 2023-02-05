// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::{info, warn};
use proto::consumer::{consumer_client::ConsumerClient, RespondRequest};
use proto::provider::{
    provider_server::Provider, GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest,
    SetResponse, SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

pub type SubscriptionMap = HashMap<String, HashSet<String>>;

#[derive(Debug, Default)]
pub struct ProviderImpl {
    pub subscription_map: Arc<Mutex<SubscriptionMap>>,
}

#[tonic::async_trait]
impl Provider for ProviderImpl {
    /// Subscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Subscribe request.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        warn!("Got a subscribe request: {:?}", request);

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
        warn!("Got an unsubscribe request: {:?}", request);

        Err(Status::unimplemented("unsubscribe has not been implemented"))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        warn!("Got a get request: {:?}", request);

        Err(Status::unimplemented("get has not been implemented"))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        warn!("Got a set request: {:?}", request);

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
        info!("Got an invoke request: {:?}", request);

        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let response_id: String = request_inner.response_id.clone();
        let consumer_uri: String = request_inner.consumer_uri;
        let payload: String = request_inner.payload;

        info!(
            "Received an invoke request from consumer URI {} for entity id {} with payload '{}'",
            &consumer_uri, &entity_id, &payload
        );

        tokio::spawn(async move {
            info!(
                "Sending an invoke respose to consumer URI {} for entity id {}",
                &consumer_uri, &entity_id
            );

            let client_result = ConsumerClient::connect(consumer_uri).await;
            if client_result.is_err() {
                return Err(Status::internal(format!("{:?}", client_result.unwrap())));
            }
            let mut client = client_result.unwrap();

            let payload: String = String::from("The send_notification response.");

            let respond_request =
                tonic::Request::new(RespondRequest { entity_id, response_id, payload });

            client.respond(respond_request).await
        });

        let response = InvokeResponse {};

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod provider_impl_tests {
    use super::*;
    use async_std::task;
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
        let result = task::block_on(provider_impl.invoke(request));
        assert!(result.is_ok());

        // Note: this test does not check that the response has successfully been sent.
    }
}
