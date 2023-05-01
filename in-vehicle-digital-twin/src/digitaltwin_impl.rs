// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

extern crate iref;

use log::Level::Debug;
use log::{debug, info, log_enabled, warn};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use proto::digitaltwin::digital_twin_server::DigitalTwin;
use proto::digitaltwin::{
    FindByIdRequest, FindByIdResponse, RegisterRequest, RegisterResponse, UnregisterRequest,
    UnregisterResponse};
use data_exchange::digitaltwin::{Entity, FindByIdRequestPayload, FindByIdResponsePayload, RegisterRequestPayload};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct DigitalTwinImpl {
    pub entity_map: Arc<RwLock<HashMap<String, Entity>>>,
}

#[tonic::async_trait]
impl DigitalTwin for DigitalTwinImpl {
    /// Find-by-id implementation.
    ///
    /// # Arguments
    /// * `request` - Find-by-id request.
    async fn find_by_id(
        &self,
        request: Request<FindByIdRequest>,
    ) -> Result<Response<FindByIdResponse>, Status> {
        let request_inner = request.into_inner();

        let payload: FindByIdRequestPayload = match serde_json::from_str(&request_inner.payload) {
            Ok(content) => content,
            Err(error) => {
                return Err(Status::internal(format!(
                    "Unexpected error with the payload: {error:?}"
                )))
            }
        };

        let entity_id = payload.id;

        info!("Received a find_by_id request for entity id {entity_id}");

        let entity: Option<Entity>;

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Entity>> = self.entity_map.read();
            entity = lock.get(&entity_id).map(|value| value.clone());          
        }

        let response_payload = FindByIdResponsePayload {
            entity
        };

        let payload = match serde_json::to_string(&response_payload) {
            Ok(content) => content,
            Err(error) => {
                return Err(Status::internal(format!(
                    "Unexpected error with the conversion to JSON for entity {entity_id}: {error}"
                )))
            }
        };

        info!("{}", payload);

        let response = FindByIdResponse { payload };

        debug!("Responded to the find_by_id request.");

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

        let payload: RegisterRequestPayload = match serde_json::from_str(&request_inner.payload) {
            Ok(content) => content,
            Err(error) => {
                return Err(Status::internal(format!(
                    "Unexpected error with the payload: {error:?}"
                )))
            }
        };

        for entity in &payload.entities {
            info!("Received a register request for the the entity:\n{}", &entity.id);

            match self.register_entity(entity.clone()) {
                Ok(_) => {
                    self.register_entity(entity.clone()).map_err(|error| return Status::internal(format!("{}", error)))?
                },
                Err(error) => return Err(Status::internal(error))
            };
        }

        let response = RegisterResponse {payload: String::from("")};

        debug!("Completed the register request.");

        Ok(Response::new(response))
    }

    /// Unregister implementation.
    ///
    /// # Arguments
    /// * `request` - Unregister request.
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        warn!("Got an unregister request: {request:?}");

        Err(Status::unimplemented("unregister has not been implemented"))
    }
}

impl DigitalTwinImpl {

    /// Register the entity.
    ///
    /// # Arguments
    /// * `entity` - The entity.
    fn register_entity(&self, entity: Entity) -> Result<(), String> {
        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Entity>> = self.entity_map.write();
            match lock.get(&entity.id) {
                Some(_) => {
                    // TODO: merge existing contents with new contents
                },
                None => {
                    lock.insert(entity.id.clone(), entity.clone());
                }
            };
        }

        if log_enabled!(Debug) {
            debug!("Registered entity {}", &entity.id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod digitaltwin_impl_tests {
    use super::*;
    use ibeji_common_test::set_dtdl_path;
    use data_exchange::digitaltwin::{Endpoint, FindByIdResponsePayload};

    #[tokio::test]
    async fn find_by_id_test() {
        set_dtdl_path();

        let mut operations = Vec::new();
        operations.push(String::from("Subscribe"));
        operations.push(String::from("Unsubscribe"));        

        let endpoint = Endpoint {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"),
            context: String::from("dtmi:sdv:Vehicle:Cabin:HAVC:AmbientAirTemperature;1"),
            operations,
        };

        let mut endpoints = Vec::new();
        endpoints.push(endpoint);

        let entity = Entity {
            digital_twin_model: String::from("dtmi:svd:vehcile;1"),
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HAVC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoints           
        };      

        let entity_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl = DigitalTwinImpl { entity_map: entity_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Entity>> = entity_map.write();
            lock.insert(entity.id.clone(), entity.clone());
        }

        let request_payload = FindByIdRequestPayload {
            id: entity.id.clone()
        };

        let request_payload_result = serde_json::to_string(&request_payload);
        assert!(request_payload_result.is_ok());

        let request = tonic::Request::new(FindByIdRequest { payload: request_payload_result.unwrap()});
        let result = digital_twin_impl.find_by_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let response_inner = response.into_inner();

        let response_payload_result: Result<FindByIdResponsePayload, serde_json::Error> = serde_json::from_str(&response_inner.payload);
        assert!(response_payload_result.is_ok());

        // assert!(!response_payload.entity.is_empty());
    }

    #[tokio::test]
    async fn register_test() {
        set_dtdl_path();

        let mut operations = Vec::new();
        operations.push(String::from("Subscribe"));
        operations.push(String::from("Unsubscribe"));        

        let endpoint = Endpoint {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"),
            context: String::from("dtmi:sdv:Vehicle:Cabin:HAVC:AmbientAirTemperature;1"),            
            operations,
        };

        let mut endpoints = Vec::new();
        endpoints.push(endpoint);

        let entity = Entity {
            digital_twin_model: String::from("dtmi:svd:vehcile;1"),
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HAVC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoints           
        };

        let mut entities = Vec::new();
        entities.push(entity.clone());  

        let entity_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl = DigitalTwinImpl { entity_map: entity_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Entity>> = entity_map.write();
            lock.insert(entity.id.clone(), entity.clone());
        }

        let register_request_payload = RegisterRequestPayload {
            entities: entities.clone()
        };

        let request_payload_result = serde_json::to_string(&register_request_payload);
        assert!(request_payload_result.is_ok());

        let request = tonic::Request::new(RegisterRequest { payload: request_payload_result.unwrap() });
        let result = digital_twin_impl.register(request).await;
        assert!(result.is_ok());

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Entity>> = entity_map.read();
            // Make sure that we populated the entity map from the contents of the DTDL.
            assert!(lock.len() == 1, "expected length was 1, actual length is {}", lock.len());
        }
    }
}
