// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use dt_model_identifiers::sdv_v1 as sdv;
use log::{debug, info, warn};
use parking_lot::{Mutex, MutexGuard};
use proto::consumer::{consumer_client::ConsumerClient, RespondRequest};
use proto::provider::{
    provider_server::Provider, GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest,
    SetResponse, SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::vehicle::Vehicle;

pub type SubscriptionMap = HashMap<String, HashSet<String>>;

#[derive(Debug, Default)]
pub struct ProviderImpl {
    pub subscription_map: Arc<Mutex<SubscriptionMap>>,
    pub vehicle: Arc<Mutex<Vehicle>>,
}

impl ProviderImpl {
    fn set_is_air_conditioning_active(
        vehicle: Arc<Mutex<Vehicle>>,
        value: &str,
    ) -> Result<(), String> {
        // This block controls the lifetime of the lock.
        {
            let mut lock: MutexGuard<Vehicle> = vehicle.lock();

            lock.is_air_conditioning_active =
                FromStr::from_str(value).map_err(|error| format!("{error:?}"))?;
        }

        Ok(())
    }

    fn show_notification(payload: &str) {
        info!("Notification: '{payload}'");
    }
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

        info!("Received a subscribe request for id {entity_id} from consumer URI {consumer_uri}");

        // This block controls the lifetime of the lock.
        {
            let mut lock: MutexGuard<HashMap<String, HashSet<String>>> =
                self.subscription_map.lock();

            let uris_option = lock.get(&entity_id);
            let mut uris = match uris_option {
                Some(get_value) => get_value.clone(),
                None => HashSet::new(),
            };
            uris.insert(consumer_uri);
            lock.insert(entity_id.clone(), uris);
        }

        debug!("Completed the subscribe request.");

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
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let value: String = request_inner.value;

        info!("Received a set request for entity id {entity_id} with value '{value}'");

        let vehicle: Arc<Mutex<Vehicle>> = self.vehicle.clone();

        tokio::spawn(async move {
            if entity_id == sdv::vehicle::cabin::hvac::is_air_conditioning_active::ID {
                let result = ProviderImpl::set_is_air_conditioning_active(vehicle.clone(), &value);
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
        debug!("Got an invoke request: {request:?}");

        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();
        let response_id: String = request_inner.response_id.clone();
        let consumer_uri: String = request_inner.consumer_uri;
        let payload: String = request_inner.payload;

        info!(
            "Received an invoke request from for entity id {entity_id} with payload '{payload}' from consumer URI {consumer_uri}"
        );

        tokio::spawn(async move {
            let mut response_payload: String = format!("Successfully invoked {entity_id}");

            if entity_id == sdv::vehicle::cabin::infotainment::hmi::show_notification::ID {
                ProviderImpl::show_notification(&payload);
            } else {
                response_payload = format!("Error: The entity id {entity_id} is not recognized.");
            }

            info!(
                "Sending an invoke response for entity id {entity_id} to consumer URI {consumer_uri} "
            );

            let client_result = ConsumerClient::connect(consumer_uri).await;
            if client_result.is_err() {
                return Err(Status::internal(format!("{:?}", client_result.unwrap_err())));
            }
            let mut client = client_result.unwrap();

            let respond_request = tonic::Request::new(RespondRequest {
                entity_id,
                response_id,
                payload: response_payload,
            });

            client.respond(respond_request).await
        });

        let response = InvokeResponse {};

        debug!("Completed the invoke request.");

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod provider_impl_tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn subscribe_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let vehicle = Arc::new(Mutex::new(Vehicle::new()));
        let provider_impl = ProviderImpl { subscription_map: subscription_map.clone(), vehicle };

        let first_id = String::from("one-id");
        let second_id = String::from("two-id");
        let first_uri = String::from("http://first.com:9000"); // Devskim: ignore DS137138
        let second_uri = String::from("http://second.com:9000"); // Devskim: ignore DS137138
        let third_uri = String::from("http://third.com:9000"); // Devskim: ignore DS137138

        let first_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: first_uri.clone(),
        });
        let first_result = provider_impl.subscribe(first_request).await;
        assert!(first_result.is_ok());

        let second_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: second_uri.clone(),
        });
        let second_result = provider_impl.subscribe(second_request).await;
        assert!(second_result.is_ok());

        let third_request = tonic::Request::new(SubscribeRequest {
            entity_id: second_id.clone(),
            consumer_uri: third_uri.clone(),
        });
        let third_result = provider_impl.subscribe(third_request).await;
        assert!(third_result.is_ok());

        // This block controls the lifetime of the lock.
        {
            let lock: MutexGuard<HashMap<String, HashSet<String>>> = subscription_map.lock();

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

    #[tokio::test]
    async fn invoke_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let vehicle = Arc::new(Mutex::new(Vehicle::new()));
        let provider_impl = ProviderImpl { subscription_map, vehicle };

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
