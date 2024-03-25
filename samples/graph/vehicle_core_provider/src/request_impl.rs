// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_graph::TargetedPayload;
// use digital_twin_model::sdv_v1 as sdv;
use log::{debug, error, info, warn};
use parking_lot::{Mutex, MutexGuard};
use samples_common::constants::digital_twin_operation;
use samples_protobuf_data_access::async_rpc::v1::request::{
    request_server::Request, AskRequest, AskResponse, NotifyRequest, NotifyResponse,
};
use samples_protobuf_data_access::async_rpc::v1::respond::{
    respond_client::RespondClient, AnswerRequest,
};
// use seat_massager_common::{status, TargetedPayload};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct InstanceData {
    pub model_id: String,
    pub description: String,
    pub serialized_value: String,
}

#[derive(Debug, Default)]
pub struct RequestState {
    pub instance_map: HashMap<String, InstanceData>,
}

#[derive(Debug, Default)]
pub struct RequestImpl {
    pub state: Arc<Mutex<RequestState>>,
}

#[tonic::async_trait]
impl Request for RequestImpl {
    /// Ask implementation.
    ///
    /// # Arguments
    /// * `request` - Ask request.
    async fn ask(
        &self,
        request: tonic::Request<AskRequest>,
    ) -> Result<tonic::Response<AskResponse>, tonic::Status> {
        let request_inner = request.into_inner();
        let respond_uri: String = request_inner.respond_uri.clone();
        let ask_id: String = request_inner.ask_id.clone();
        let payload: String = request_inner.payload.clone();

        info!("Received an ask request");
        info!("  respond_uri: {respond_uri}");
        info!("  ask_id: {ask_id}");

        // Deserialize the targeted payload.
        let targeted_payload_json: TargetedPayload = serde_json::from_str(&payload).unwrap();

        info!("  instance_id: {}", targeted_payload_json.instance_id);
        info!("  member_path: {}", targeted_payload_json.member_path);
        info!("  operation: {}", targeted_payload_json.operation);

        // Extract the type_id from the request payload.
        // let type_id_json: serde_json::Value = request_payload_json.get("@type").unwrap().clone();
        // let type_id: String = serde_json::from_value(type_id_json.clone()).unwrap();

        // Check to make sure that the type_id is for a perform_request request.
        if targeted_payload_json.operation != digital_twin_operation::GET {
            return Err(tonic::Status::invalid_argument(format!(
                "Unexpected operation '{}'",
                targeted_payload_json.operation
            )));
        }

        if !targeted_payload_json.payload.is_empty() {
            return Err(tonic::Status::invalid_argument(format!(
                "Unexpected payload, it should be empty, not '{}'",
                targeted_payload_json.payload
            )));
        }

        let state: Arc<Mutex<RequestState>> = self.state.clone();

        // Asynchronously perform the step.
        tokio::spawn(async move {
            let instance_data: InstanceData = {
                let lock: MutexGuard<RequestState> = state.lock();
                match lock.instance_map.get(&targeted_payload_json.instance_id) {
                    Some(instance_data) => instance_data.clone(),
                    None => {
                        error!(
                            "Instance not found for instance id '{}'",
                            targeted_payload_json.instance_id
                        );
                        return;
                    }
                }
            };

            let response_payload_json = instance_data.serialized_value.clone();

            let client_result = RespondClient::connect(respond_uri).await;
            if let Err(error_message) = client_result {
                error!("Unable to connect due to {error_message}");
                return;
            }
            let mut client = client_result.unwrap();

            // Serilaize the response payload.
            // let response_payload_json: String =
            //    serde_json::to_string_pretty(&response_payload).unwrap();

            let answer_request =
                tonic::Request::new(AnswerRequest { ask_id, payload: response_payload_json });

            // Send the answer.
            let response = client.answer(answer_request).await;
            if let Err(status) = response {
                error!("Answer failed: {status:?}");
            }
        });

        debug!("Completed the ask request.");

        Ok(tonic::Response::new(AskResponse {}))
    }

    /// Notify implementation.
    ///
    /// # Arguments
    /// * `request` - Notify request.
    async fn notify(
        &self,
        request: tonic::Request<NotifyRequest>,
    ) -> Result<tonic::Response<NotifyResponse>, tonic::Status> {
        warn!("Got a notify request: {request:?}");

        Err(tonic::Status::unimplemented("notify has not been implemented"))
    }
}
