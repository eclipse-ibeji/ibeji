// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

extern crate iref;

use core_protobuf_data_access::module::digital_twin_registry::v1::digital_twin_registry_server::DigitalTwinRegistry;
use core_protobuf_data_access::module::digital_twin_registry::v1::{
    EntityAccessInfo, FindByInstanceIdRequest, FindByInstanceIdResponse, FindByModelIdRequest,
    FindByModelIdResponse, RegisterRequest, RegisterResponse,
};
use log::{debug, info};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct DigitalTwinRegistryImpl {
    /// Entity access info map.
    pub entity_access_info_map: Arc<RwLock<HashMap<String, Vec<EntityAccessInfo>>>>,
}

#[tonic::async_trait]
impl DigitalTwinRegistry for DigitalTwinRegistryImpl {
    /// Find by model id implementation.
    ///
    /// # Arguments
    /// * `request` - Find by model id request.
    async fn find_by_model_id(
        &self,
        request: Request<FindByModelIdRequest>,
    ) -> Result<Response<FindByModelIdResponse>, Status> {
        let model_id = request.into_inner().model_id;

        debug!("Received a find_by_model_id request for entity id {model_id}");

        let entity_access_info_list;

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                self.entity_access_info_map.read();
            entity_access_info_list = lock.get(&model_id).cloned();
        }

        if entity_access_info_list.is_none() {
            return Err(Status::not_found("Unable to find any entities with model id {model_id}"));
        }

        let response =
            FindByModelIdResponse { entity_access_info_list: entity_access_info_list.unwrap() };

        debug!("Completed the find_by_model_id request.");

        Ok(Response::new(response))
    }

    /// Find by instance id implementation.
    ///
    /// # Arguments
    /// * `request` - Find by instamce id request.
    async fn find_by_instance_id(
        &self,
        request: Request<FindByInstanceIdRequest>,
    ) -> Result<Response<FindByInstanceIdResponse>, Status> {
        let instance_id = request.into_inner().instance_id;

        debug!("Received a find_by_instance_id request for instance id {instance_id}");

        let mut matching_entity_access_info_list = Vec::<EntityAccessInfo>::new();

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                self.entity_access_info_map.read();
            for entity_access_info_list in lock.values() {
                for entity_access_info in entity_access_info_list {
                    if entity_access_info.instance_id == instance_id {
                        matching_entity_access_info_list.push(entity_access_info.clone());
                    }
                }
            }
        }

        if matching_entity_access_info_list.is_empty() {
            return Err(Status::not_found(
                "Unable to find any entities with instance id {instance_id}",
            ));
        }

        let response =
            FindByInstanceIdResponse { entity_access_info_list: matching_entity_access_info_list };

        debug!("Completed the find_by_instance_id request.");

        Ok(Response::new(response))
    }

    /// Register implementation.
    ///
    /// # Arguments
    /// * `request` - Publish request.
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request_inner = request.into_inner();

        for entity_access_info in &request_inner.entity_access_info_list {
            self.register_entity(&entity_access_info)?;

            info!(
                "Registered the entity with provider id: {} instance id: {} model id: {}",
                entity_access_info.provider_id,
                entity_access_info.instance_id,
                entity_access_info.model_id
            );
        }

        let response = RegisterResponse {};

        debug!("Completed the register request.");

        Ok(Response::new(response))
    }
}

impl DigitalTwinRegistryImpl {
    /// Register an entity.
    ///
    /// # Arguments
    /// * `entity` - The entity.
    fn register_entity(&self, entity_access_info: &EntityAccessInfo) -> Result<(), Status> {
        if entity_access_info.provider_id.is_empty() {
            return Err(Status::invalid_argument("Provider id is required"));
        }

        if entity_access_info.model_id.is_empty() {
            return Err(Status::invalid_argument("Model id is required"));
        }

        if entity_access_info.instance_id.is_empty() {
            return Err(Status::invalid_argument("Instance id is required"));
        }

        if entity_access_info.protocol.is_empty() {
            return Err(Status::invalid_argument("Protocol is required"));
        }

        if entity_access_info.uri.is_empty() {
            return Err(Status::invalid_argument("Uri is required"));
        }

        if entity_access_info.operations.is_empty() {
            return Err(Status::invalid_argument("Operations is required"));
        }

        // This block controls the lifetime of the lock.
        {
            // Note: the context is optional.

            let mut lock: RwLockWriteGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                self.entity_access_info_map.write();
            let get_result = lock.get(&entity_access_info.model_id);
            match get_result {
                Some(_) => {
                    info!(
                        "Registered another entity access info for entity {}",
                        &entity_access_info.model_id
                    );
                    lock.get_mut(&entity_access_info.model_id)
                        .unwrap()
                        .push(entity_access_info.clone());
                }
                None => {
                    info!("Registered entity {}", &entity_access_info.model_id);
                    lock.insert(
                        entity_access_info.model_id.clone(),
                        vec![entity_access_info.clone()],
                    );
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod digital_twin_registry_impl_tests {
    use super::*;

    #[tokio::test]
    async fn find_by_model_id_test() {
        let operations = vec![String::from("Subscribe"), String::from("Unsubscribe")];

        let entity_access_info = EntityAccessInfo {
            provider_id: String::from("test-provider"),
            instance_id: String::from("1234567890"),
            model_id: String::from("dtmi:sdv:hvac:ambient_air_temperature;1"),
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from(""),
            operations,
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_registry_impl =
            DigitalTwinRegistryImpl { entity_access_info_map: entity_access_info_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                entity_access_info_map.write();
            lock.insert(entity_access_info.model_id.clone(), vec![entity_access_info.clone()]);
        }

        let request = tonic::Request::new(FindByModelIdRequest {
            model_id: String::from("dtmi:sdv:hvac:ambient_air_temperature;1"),
        });
        let result = digital_twin_registry_impl.find_by_model_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let response_inner = response.into_inner();

        assert!(response_inner.entity_access_info_list.len() == 1);

        let response_entity_access_info = response_inner.entity_access_info_list[0].clone();

        assert_eq!(response_entity_access_info.model_id, "dtmi:sdv:hvac:ambient_air_temperature;1");
        assert_eq!(
            response_entity_access_info.uri,
            "http://[::1]:40010" // Devskim: ignore DS137138
        );
    }

    #[tokio::test]
    async fn find_by_instance_id_test() {
        let operations = vec![String::from("Subscribe"), String::from("Unsubscribe")];

        let entity_access_info = EntityAccessInfo {
            provider_id: String::from("test-provider"),
            instance_id: String::from("1234567890"),
            model_id: String::from("dtmi:sdv:hvac:ambient_air_temperature;1"),
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from(""),
            operations,
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_registry_impl =
            DigitalTwinRegistryImpl { entity_access_info_map: entity_access_info_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                entity_access_info_map.write();
            lock.insert(entity_access_info.model_id.clone(), vec![entity_access_info.clone()]);
        }

        let request = tonic::Request::new(FindByInstanceIdRequest {
            instance_id: String::from("1234567890"),
        });
        let result = digital_twin_registry_impl.find_by_instance_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let response_inner = response.into_inner();

        assert!(response_inner.entity_access_info_list.len() == 1);

        let response_entity_access_info = response_inner.entity_access_info_list[0].clone();

        assert_eq!(response_entity_access_info.instance_id, "1234567890");
        assert_eq!(
            response_entity_access_info.uri,
            "http://[::1]:40010" // Devskim: ignore DS137138
        );
    }

    #[tokio::test]
    async fn register_test() {
        let entity_access_info = EntityAccessInfo {
            provider_id: String::from("test-provider"),
            instance_id: String::from("1234567890"),
            model_id: String::from("dtmi:sdv:hvac:ambient_air_temperature;1"),
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from(""),
            operations: vec![String::from("Subscribe"), String::from("Unsubscribe")],
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_registry_impl =
            DigitalTwinRegistryImpl { entity_access_info_map: entity_access_info_map.clone() };

        let request = tonic::Request::new(RegisterRequest {
            entity_access_info_list: vec![entity_access_info],
        });
        let result = digital_twin_registry_impl.register(request).await;
        assert!(result.is_ok(), "register result is not okay: {result:?}");

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Vec<EntityAccessInfo>>> =
                entity_access_info_map.read();
            // Make sure that we populated the entity map from the contents of the DTDL.
            assert_eq!(lock.len(), 1, "expected length was 1, actual length is {}", lock.len());
        }
    }
}
