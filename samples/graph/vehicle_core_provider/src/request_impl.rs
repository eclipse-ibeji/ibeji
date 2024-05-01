// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_graph::TargetedPayload;
use log::{info, warn};
use parking_lot::{Mutex, MutexGuard};
use samples_common::constants::digital_twin_operation;
use samples_protobuf_data_access::async_rpc::v1::request::{
    request_server::Request, AskRequest, AskResponse, NotifyRequest, NotifyResponse,
};
use samples_protobuf_data_access::async_rpc::v1::respond::{
    respond_client::RespondClient, AnswerRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

/// Instance data.
#[derive(Clone, Debug, Default)]
pub struct InstanceData {
    /// Model Id.
    pub model_id: String,
    /// Description.
    pub description: String,
    /// Serialized value (using JSON-LD as a string).
    pub serialized_value: String,
}

/// Provider state.
#[derive(Debug, Default)]
pub struct ProviderState {
    /// Maps an instance id to its associated instance data.
    pub instance_map: HashMap<String, InstanceData>,
}

#[derive(Debug, Default)]
pub struct RequestImpl {
    /// Provider state.
    pub provider_state: Arc<Mutex<ProviderState>>,
}

/// The implementation for the Request interface, which is used to handle requests from the consumer.
impl RequestImpl {
    ///
    const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;

    ///
    const MAX_RETRIES: usize = 100;

    /// Get implementation.
    ///
    /// # Arguments
    /// * `respond_uri` - Respond URI.
    /// * `ask_id` - Ask Id.
    /// * `targeted_payload` - Targeted payload.
    async fn get(
        &self,
        respond_uri: String,
        ask_id: String,
        targeted_payload: TargetedPayload,
    ) -> Result<tonic::Response<AskResponse>, tonic::Status> {
        if !targeted_payload.payload.is_empty() {
            return Err(tonic::Status::invalid_argument(
                "Unexpected payload, it should be empty".to_string(),
            ));
        }

        let provider_state: Arc<Mutex<ProviderState>> = self.provider_state.clone();

        // Define a retry strategy.
        let retry_strategy = ExponentialBackoff::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)
            .map(jitter) // add jitter to delays
            .take(Self::MAX_RETRIES);

        // Asynchronously perform the get.
        tokio::spawn(async move {
            // Retrieve the instance's value (it will be represented as a JSON string).
            let instance_value: String = {
                let instance_data: InstanceData = {
                    let lock: MutexGuard<ProviderState> = provider_state.lock();
                    match lock.instance_map.get(&targeted_payload.instance_id) {
                        Some(instance_data) => instance_data.clone(),
                        None => {
                            return Err(format!(
                                "Instance not found for instance id '{}'",
                                targeted_payload.instance_id
                            ));
                        }
                    }
                };

                instance_data.serialized_value.clone()
            };

            // Send the answer to the consumer.
            Retry::spawn(retry_strategy, || async {
                // Connect to the consumer.
                let mut client = RespondClient::connect(respond_uri.clone())
                    .await
                    .map_err(|err_msg| format!("Unable to connect due to: {err_msg}"))?;

                // Prepare the answer request.
                let answer_request = tonic::Request::new(AnswerRequest {
                    ask_id: ask_id.clone(),
                    payload: instance_value.clone(),
                });

                // Send the answer to the consumer.
                client
                    .answer(answer_request)
                    .await
                    .map_err(|status| format!("Answer failed: {status:?}"))
            })
            .await
        });

        Ok(tonic::Response::new(AskResponse {}))
    }
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
        let ask_request = request.into_inner();

        info!("Received an ask request:");
        info!("  respond_uri: {}", ask_request.respond_uri);
        info!("  ask_id: {}", ask_request.ask_id);

        // Deserialize the targeted payload.
        let targeted_payload_json: TargetedPayload =
            serde_json::from_str(&ask_request.payload).unwrap();

        info!("  instance_id: {}", targeted_payload_json.instance_id);
        info!("  member_path: {}", targeted_payload_json.member_path);
        info!("  operation: {}", targeted_payload_json.operation);

        if targeted_payload_json.operation == digital_twin_operation::GET {
            self.get(ask_request.respond_uri, ask_request.ask_id, targeted_payload_json).await
        } else {
            Err(tonic::Status::invalid_argument(format!(
                "Unexpected operation '{}'.  Expected '{}'.",
                targeted_payload_json.operation,
                digital_twin_operation::GET
            )))
        }
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
