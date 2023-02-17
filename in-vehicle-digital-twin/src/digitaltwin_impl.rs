// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

extern crate iref;

use dtdl_parser::model_parser::ModelParser;
use log::Level::Debug;
use log::{debug, info, log_enabled, warn};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use proto::digitaltwin::digital_twin_server::DigitalTwin;
use proto::digitaltwin::{
    FindByIdRequest, FindByIdResponse, RegisterRequest, RegisterResponse, UnregisterRequest,
    UnregisterResponse,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct DigitalTwinImpl {
    pub entity_map: Arc<RwLock<HashMap<String, Value>>>,
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
        let entity_id = request_inner.entity_id;

        debug!("Received a find_by_id request for entity id {entity_id}");

        let dtdl;

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Value>> = self.entity_map.read();
            let val_option = lock.get(&entity_id);
            let val = match val_option {
                Some(v) => v,
                None => {
                    return Err(Status::not_found(format!(
                        "Unable to find the DTDL for entity id {entity_id}"
                    )))
                }
            };

            dtdl = match serde_json::to_string_pretty(&val) {
                Ok(content) => content,
                Err(error) => {
                    return Err(Status::internal(format!(
                        "Unexpected error with the DTDL for entity id {entity_id}: {error:?}"
                    )))
                }
            };
        }

        let response = FindByIdResponse { dtdl };

        info!(
            "Responded to the find_by_id request for entity id {entity_id} with the requested DTDL."
        );

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
        let dtdl = request_inner.dtdl;

        let register_each_one_result = self.register_each_one(&dtdl);
        if let Err(error) = register_each_one_result {
            return Err(Status::internal(error));
        }

        let response = RegisterResponse {};

        info!("Completed the resgister request.");

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
    /// This function assumes that an array of resources has been provided and that each resource in the array needs to be registered.
    ///
    /// # Arguments
    /// * `dtdl` - The DTDL for the array.
    #[allow(unused_variables)]
    fn register_each_one(&self, dtdl: &str) -> Result<(), String> {
        let doc: Value = match serde_json::from_str(dtdl) {
            Ok(json) => json,
            Err(error) => return Err(format!("Failed to parse the DTDL due to: {error:?}")),
        };

        match doc {
            Value::Array(array) => {
                for v in array.iter() {
                    self.register_one(v)?
                }
            }
            _ => return Err(String::from("An unexpected item was encountered in the DTDL.")),
        };

        Ok(())
    }

    /// Register the resource specified in the the JSON doc.
    ///
    /// # Arguments
    /// * `doc` - The JSON doc that specifies the entity.
    fn register_one(&self, doc: &Value) -> Result<(), String> {
        let dtdl = match serde_json::to_string_pretty(&doc) {
            Ok(content) => content,
            Err(error) => {
                return Err(format!("Failed to make the DTDL pretty due to: : {error:?}"))
            }
        };

        let mut parser = ModelParser::new();
        let json_texts = vec![dtdl];

        let model_dict_result = parser.parse(&json_texts);
        if let Err(error) = model_dict_result {
            return Err(format!("Failed to parse the DTDL due to: {error:?}"));
        }
        let model_dict = model_dict_result.unwrap();

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Value>> = self.entity_map.write();
            for id in model_dict.keys() {
                lock.insert(id.to_string(), doc.clone());
            }
        }

        if log_enabled!(Debug) {
            for id in model_dict.keys() {
                debug!("Registered DTDL for id {id}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod digitaltwin_impl_tests {
    use super::*;
    use ibeji_common::find_full_path;
    use ibeji_common_test::set_dtdl_path;
    use std::fs;
    use std::path::Path;

    fn retrieve_dtdl(file_path: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        let read_result = fs::read_to_string(path);
        match read_result {
            Ok(contents) => Ok(contents),
            Err(error) => Err(format!("Unable to retrieve the DTDL due to: {error:?}")),
        }
    }

    #[tokio::test]
    async fn find_by_id_test() {
        set_dtdl_path();

        // Note: We can use any valid JSON.  We'll use samples/remotely_accessible_resource.json.
        let dtdl_path_result = find_full_path("samples/remotely_accessible_resource.json");
        assert!(dtdl_path_result.is_ok());
        let dtdl_path = dtdl_path_result.unwrap();
        let dtdl_result = retrieve_dtdl(&dtdl_path);
        assert!(dtdl_result.is_ok());
        let dtdl = dtdl_result.unwrap();

        let dtdl_json_result = serde_json::from_str(&dtdl);
        assert!(dtdl_json_result.is_ok());
        let dtdl_json = dtdl_json_result.unwrap();

        let entity_id = String::from("dtmi::some_id");

        let entity_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl = DigitalTwinImpl { entity_map: entity_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, Value>> = entity_map.write();
            lock.insert(entity_id.clone(), dtdl_json);
        }

        let request = tonic::Request::new(FindByIdRequest { entity_id });
        let result = digital_twin_impl.find_by_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let dtdl = response.into_inner().dtdl;
        assert!(!dtdl.is_empty());
    }

    #[tokio::test]
    async fn register_test() {
        set_dtdl_path();

        let entity_map = Arc::new(RwLock::new(HashMap::new()));
        let digital_twin_impl = DigitalTwinImpl { entity_map: entity_map.clone() };

        let dtdl_path_result = find_full_path("samples/demo_resources.json");
        assert!(dtdl_path_result.is_ok());
        let dtdl_path = dtdl_path_result.unwrap();
        let dtdl_result = retrieve_dtdl(&dtdl_path);
        assert!(dtdl_result.is_ok());
        let dtdl = dtdl_result.unwrap();

        let request = tonic::Request::new(RegisterRequest { dtdl });
        let result = digital_twin_impl.register(request).await;
        assert!(result.is_ok());

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, Value>> = entity_map.read();
            // Make sure that we populated the entity map from the contents of the DTDL.
            assert!(lock.len() == 13, "expected length was 13, actual length is {}", lock.len());
        }
    }
}
