// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

extern crate iref;

use log::Level::Debug;
use log::{debug, info, log_enabled, warn};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use proto::digital_twin::digital_twin_server::DigitalTwin;
use proto::digital_twin::{
    EntityAccessInfo, FindByIdRequest, FindByIdResponse, RegisterRequest, RegisterResponse,
    UnregisterRequest, UnregisterResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct DigitalTwinImpl {
    pub entity_access_info_map: Arc<RwLock<HashMap<String, EntityAccessInfo>>>,
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
        let entity_id = request.into_inner().id;

        info!("Received a find_by_id request for entity id {entity_id}");

        let entity_access_info;

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, EntityAccessInfo>> =
                self.entity_access_info_map.read();
            entity_access_info = lock.get(&entity_id).cloned();
        }

        info!("{entity_access_info:?}");

        if entity_access_info.is_none() {
            return Err(Status::not_found("Unable to find the entity with id {entity_id}"));
        }

        let response = FindByIdResponse { entity_access_info };

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

        for entity_access_info in &request_inner.entity_access_info_list {
            info!("Received a register request for the the entity:\n{}", entity_access_info.id);

            self.register_entity(entity_access_info.clone()).map_err(Status::internal)?;
        }

        let response = RegisterResponse {};

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
    fn register_entity(&self, entity_access_info: EntityAccessInfo) -> Result<(), String> {
        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, EntityAccessInfo>> =
                self.entity_access_info_map.write();
            match lock.get(&entity_access_info.id) {
                Some(_) => {
                    // TODO: merge existing contents with new contents
                }
                None => {
                    lock.insert(entity_access_info.id.clone(), entity_access_info.clone());
                }
            };
        }

        if log_enabled!(Debug) {
            debug!("Registered entity {}", &entity_access_info.id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod digitaltwin_impl_tests {
    use super::*;
    use ibeji_common_test::set_dtdl_path;
    use proto::digital_twin::EndpointInfo;

    #[tokio::test]
    async fn find_by_id_test() {
        set_dtdl_path();

        let operations = vec![String::from("Subscribe"), String::from("Unsubscribe")];

        let endpoint_info = EndpointInfo {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            operations,
        };

        let entity_access_info = EntityAccessInfo {
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoint_info_list: vec![endpoint_info],
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl =
            DigitalTwinImpl { entity_access_info_map: entity_access_info_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, EntityAccessInfo>> =
                entity_access_info_map.write();
            lock.insert(entity_access_info.id.clone(), entity_access_info.clone());
        }

        let request = tonic::Request::new(FindByIdRequest {
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
        });
        let result = digital_twin_impl.find_by_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let response_inner = response.into_inner();

        assert!(response_inner.entity_access_info.is_some());

        assert_eq!(
            response_inner.entity_access_info.unwrap().id,
            "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"
        );

        // TODO: add check
    }

    #[tokio::test]
    async fn register_test() {
        set_dtdl_path();

        let endpoint_info = EndpointInfo {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            operations: vec![String::from("Subscribe"), String::from("Unsubscribe")],
        };

        let entity_access_info = EntityAccessInfo {
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoint_info_list: vec![endpoint_info],
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl =
            DigitalTwinImpl { entity_access_info_map: entity_access_info_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, EntityAccessInfo>> =
                entity_access_info_map.write();
            lock.insert(entity_access_info.id.clone(), entity_access_info.clone());
        }

        let request = tonic::Request::new(RegisterRequest {
            entity_access_info_list: vec![entity_access_info],
        });
        let result = digital_twin_impl.register(request).await;
        assert!(result.is_ok());

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, EntityAccessInfo>> =
                entity_access_info_map.read();
            // Make sure that we populated the entity map from the contents of the DTDL.
            assert_eq!(lock.len(), 1, "expected length was 1, actual length is {}", lock.len());
        }
    }
}
