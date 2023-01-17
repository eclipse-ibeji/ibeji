// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::info;
use proto::provider::{
    provider_server::Provider, GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse, SubscribeRequest,
    SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse
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
        let id: String = request_inner.id.clone();
        let uri: String = request_inner.uri;

        info!("Received a subscribe request from URI {} for id {}", &uri, &id);

        let mut lock: MutexGuard<HashMap<String, HashSet<String>>> =
            self.subscription_map.lock().unwrap();
        let get_result = lock.get(&id);
        if let Some(get_value) = get_result {
            let mut uris = get_value.clone();
            uris.insert(uri);
            lock.insert(id, uris);
        } else {
            let mut uris = HashSet::new();
            uris.insert(uri);
            lock.insert(id, uris);
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
        info!("Got an unsubscribe request: {:?}", request);
        // TODO - provide unsubscribe functionality
        let response = UnsubscribeResponse {};

        Ok(Response::new(response))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        info!("Got a get request: {:?}", request);
        // TODO - provide get functionality
        let response = GetResponse {};

        Ok(Response::new(response))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        info!("Got a set request: {:?}", request);
        // TODO - provide set functionality
        let response = SetResponse {};

        Ok(Response::new(response))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(&self, request: Request<InvokeRequest>) -> Result<Response<InvokeResponse>, Status> {
        info!("Got an invoke request: {:?}", request);
        // TODO - provide invoke functionality
        let response = InvokeResponse {};

        Ok(Response::new(response))
    }    
}

#[cfg(test)]
mod provider_impl_tests {
    use super::*;
    use async_std::task;

    #[test]
    fn subscribe_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };

        let first_id = String::from("one-id");
        let second_id = String::from("two-id");
        let first_uri = String::from("http://first.com:9000"); // Devskim: ignore DS137138
        let second_uri = String::from("http://second.com:9000"); // Devskim: ignore DS137138
        let third_uri = String::from("http://third.com:9000"); // Devskim: ignore DS137138

        let first_request =
            tonic::Request::new(SubscribeRequest { id: first_id.clone(), uri: first_uri.clone() });
        let first_result = task::block_on(provider_impl.subscribe(first_request));
        assert!(first_result.is_ok());

        let second_request =
            tonic::Request::new(SubscribeRequest { id: first_id.clone(), uri: second_uri.clone() });
        let second_result = task::block_on(provider_impl.subscribe(second_request));
        assert!(second_result.is_ok());

        let third_request =
            tonic::Request::new(SubscribeRequest { id: second_id.clone(), uri: third_uri.clone() });
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
}
