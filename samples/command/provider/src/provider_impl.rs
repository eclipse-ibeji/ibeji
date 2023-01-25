// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::{info, warn};
use proto::consumer::{consumer_client::ConsumerClient, RespondRequest};
use proto::provider::{
    provider_server::Provider, GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest,
    SetResponse, SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};
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
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let consumer_uri: String = request_inner.consumer_uri;

        info!(
            "Received a subscribe request from consumer URI {} for entity id {}",
            &consumer_uri, &entity_id
        );

        let mut lock: MutexGuard<HashMap<String, HashSet<String>>> =
            self.subscription_map.lock().unwrap();
        let get_result = lock.get(&entity_id);
        if let Some(get_value) = get_result {
            let mut uris = get_value.clone();
            uris.insert(consumer_uri);
            lock.insert(entity_id, uris);
        } else {
            let mut uris = HashSet::new();
            uris.insert(consumer_uri);
            lock.insert(entity_id, uris);
        }

        info!("Completed subscription.");

        let response = SubscribeResponse {};

        Ok(Response::new(response))
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

    #[test]
    fn subscribe_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };

        let first_id = String::from("one-id");
        let second_id = String::from("two-id");
        let first_uri = String::from("http://first.com:9000"); // Devskim: ignore DS137138
        let second_uri = String::from("http://second.com:9000"); // Devskim: ignore DS137138
        let third_uri = String::from("http://third.com:9000"); // Devskim: ignore DS137138

        let first_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: first_uri.clone(),
        });
        let first_result = task::block_on(provider_impl.subscribe(first_request));
        assert!(first_result.is_ok());

        let second_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: second_uri.clone(),
        });
        let second_result = task::block_on(provider_impl.subscribe(second_request));
        assert!(second_result.is_ok());

        let third_request = tonic::Request::new(SubscribeRequest {
            entity_id: second_id.clone(),
            consumer_uri: third_uri.clone(),
        });
        let third_result = task::block_on(provider_impl.subscribe(third_request));
        assert!(third_result.is_ok());

        let lock: MutexGuard<HashMap<String, HashSet<String>>> = subscription_map.lock().unwrap();

        let first_get_result = lock.get(&first_id);
        assert!(first_get_result.is_some());
        let first_value = first_get_result.unwrap();
        assert!(first_value.len() == 2);
        assert!(first_value.contains(&first_uri));
        assert!(first_value.contains(&second_uri));

        let second_get_result = lock.get(&second_id);
        assert!(second_get_result.is_some());
        let second_value = second_get_result.unwrap();
        assert!(second_value.len() == 1);
        assert!(second_value.contains(&third_uri));
    }

    #[tokio::test]
    async fn invoke_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let provider_impl = ProviderImpl { subscription_map };

        let entity_id = String::from("one-id");
        let consumer_uri = String::from("bogus uri");

        let response_id = Uuid::new_v4().to_string();
        let payload = String::from("some-payload");

        let request = tonic::Request::new(InvokeRequest {
            entity_id,
            consumer_uri,
            response_id,
            payload,
        });
        let result = task::block_on(provider_impl.invoke(request));
        assert!(result.is_ok());

        // Note: this test does not check that the response has successfully been sent.
    }
}
