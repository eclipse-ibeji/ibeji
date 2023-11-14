// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use log::{debug, info};
use samples_protobuf_data_access::tutorial_grpc::v1::digital_twin_provider_tutorial_server::DigitalTwinProviderTutorial;
use samples_protobuf_data_access::tutorial_grpc::v1::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct ProviderImpl {}

#[tonic::async_trait]
impl DigitalTwinProviderTutorial for ProviderImpl {
    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request_inner = request.into_inner();
        let entity_id: String = request_inner.entity_id.clone();

        let value = match entity_id.as_str() {
            sdv::hvac::ambient_air_temperature::ID => "42",
            sdv::hvac::is_air_conditioning_active::ID => "true",
            _ => "NULL"
        };

        let get_response = GetResponse { property_value: String::from(value) };

        Ok(Response::new(get_response))
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
        let payload: String = request_inner.payload;

        info!("Received an invoke request from for entity id {entity_id} with payload '{payload}'");

        let response_message: String = if entity_id == sdv::hmi::show_notification::ID {
            format!("Displaying notification '{payload}'")
        } else {
            format!("Error notification: The entity id {entity_id} is not recognized.")
        };

        info!("Sending an invoke response for entity id {entity_id}");

        let response = InvokeResponse { response: response_message };

        Ok(Response::new(response))
    }
}
