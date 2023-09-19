// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use core_protobuf_data_access::invehicle_digital_twin::{self, v1::RegisterRequest};
use log::{info, warn};
use parking_lot::RwLock;
use prost::Message;
use std::{error::Error, sync::Arc, collections::HashMap};

use common::grpc_interceptor::GrpcInterceptor;

use crate::managed_subscribe::managed_subscribe_store::{CallbackInfo, EntityMetadata, ManagedSubscribeStore};

/// Interceptor for injecting a managed subscribe endpoint for providers.
#[derive(Clone)]
pub struct ManagedSubscribeInterceptor {
    extension_uri: String,
    extension_store: Arc<RwLock<ManagedSubscribeStore>>,
}

impl ManagedSubscribeInterceptor {
    const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "InvehicleDigitalTwin";
    const REGISTER_METHOD_NAME: &str = "Register";
    const MANAGED_SUBSCRIBE_OPERATION: &str = "ManagedSubscribe";

    pub fn new(extension_uri: &str, extension_store: Arc<RwLock<ManagedSubscribeStore>>) -> Self {
        ManagedSubscribeInterceptor {
            extension_uri: extension_uri.to_string(),
            extension_store,
        }
    }
}

impl GrpcInterceptor for ManagedSubscribeInterceptor {
    /// Is this interceptor applicable?
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    fn is_applicable(&self, service_name: &str, method_name: &str) -> bool {
        service_name == Self::INVEHICLE_DIGITAL_TWIN_SERVICE_NAME
            && method_name == Self::REGISTER_METHOD_NAME
    }

    /// Indicates that the request must be handled.
    fn must_handle_request(&self) -> bool {
        true
    }

    /// Indicates that the response must be handled.
    fn must_handle_response(&self) -> bool {
        false
    }

    /// Handle request. Return the new request.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The request's protobuf messages as bytes.
    fn handle_request(
        &self,
        _service_name: &str,
        _method_name: &str,
        protobuf_message_bytes: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let register_request: invehicle_digital_twin::v1::RegisterRequest =
            Message::decode(&protobuf_message_bytes[..]).unwrap();

        info!("register_request = {:?}", register_request);

        let mut entities = register_request.entity_access_info_list;

        for i in 0..entities.len() {
            let entity_id = entities[i].id.clone();

            let mut endpoints = entities[i].endpoint_info_list.clone();

            for j in 0..endpoints.len() {
                if endpoints[j].operations.contains(&Self::MANAGED_SUBSCRIBE_OPERATION.to_string()) {
                    let entity_callback = endpoints[j].uri.clone();
                    let callback_protocol = endpoints[j].protocol.clone();

                    // Set endpoint information to the managed subscribe extension.
                    endpoints[j].uri = self.extension_uri.clone();
                    endpoints[j].protocol = "grpc".to_string();
                    endpoints[j].operations = vec![Self::MANAGED_SUBSCRIBE_OPERATION.to_string()];
                    endpoints[j].context = "GetSubscriptionInfo".to_string();

                    // Pass the callback with relevant endpoint information to extension.
                    let entity_metadata = EntityMetadata {
                        callback: CallbackInfo {
                            uri: entity_callback.clone(),
                            protocol: callback_protocol.clone(),
                        },
                        topics: HashMap::new(),
                    };

                    info!("add entity metadata with id: {entity_id}, callback: {entity_callback}");
                    {
                        let mut store_lock = self.extension_store.write();
                        store_lock.add_entity(&entity_id, entity_metadata);
                    }

                    break;
                }
            }

            // Add the modified endpoint info list back to the entity access info object.
            entities[i].endpoint_info_list = endpoints;
        }

        // Construct modified register request.
        let updated_register_request = RegisterRequest {
            entity_access_info_list: entities,
        };

        let mut new_protobuf_message_buf: Vec<u8> = Vec::new();
        new_protobuf_message_buf.reserve(updated_register_request.encoded_len());
        updated_register_request.encode(&mut new_protobuf_message_buf).unwrap();
        Ok(Bytes::from(new_protobuf_message_buf))
    }

    /// Handle response. Return the new response.
    ///
    /// # Arguments
    /// * `service_name` - The gRPC call's service name.
    /// * `method_name` - The gRPC call's method name.
    /// * `protobuf_message_bytes` - The response's protobuf messages as bytes.
    fn handle_response(
        &self,
        _service_name: &str,
        _method_name: &str,
        protobuf_message_bytes: Bytes,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        warn!("handle_response is unimplemented");

        Ok(protobuf_message_bytes)
    }
}
