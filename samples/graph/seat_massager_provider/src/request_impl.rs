// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_graph::TargetedPayload;
use digital_twin_model::sdv_v1 as sdv;
use log::{debug, info, warn};
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
    /// Model ID.
    pub model_id: String,
    /// Description.
    pub description: String,
    /// Serialized value (using JSON-LD as a string).
    pub serialized_value: String,
}

/// The provider's state.
#[derive(Debug, Default)]
pub struct ProviderState {
    /// Maps an instance id to its associated instance data.
    pub instance_map: HashMap<String, InstanceData>,
}

#[derive(Debug, Default)]
pub struct RequestImpl {
    pub provider_state: Arc<Mutex<ProviderState>>,
}

impl RequestImpl {
    const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;
    const MAX_RETRIES: usize = 100;

    /// Get implementation.
    ///
    /// # Arguments
    /// * `respond_uri` - Respond URI.
    /// * `ask_id` - Ask ID.
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

        let state: Arc<Mutex<ProviderState>> = self.provider_state.clone();

        // Define a retry strategy.
        let retry_strategy = ExponentialBackoff::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)
            .map(jitter) // add jitter to delays
            .take(Self::MAX_RETRIES);

        // Asynchronously perform the get.
        tokio::spawn(async move {
            // Get the answer's payload.
            let answer_payload: String = {
                let instance_data: InstanceData = {
                    let lock: MutexGuard<ProviderState> = state.lock();
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
                    payload: answer_payload.clone(),
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

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `respond_uri` - Respond URI.
    /// * `ask_id` - Ask ID.
    /// * `targeted_payload` - Targeted payload.
    async fn invoke(
        &self,
        respond_uri: String,
        ask_id: String,
        targeted_payload: TargetedPayload,
    ) -> Result<tonic::Response<AskResponse>, tonic::Status> {
        if targeted_payload.payload.is_empty() {
            return Err(tonic::Status::invalid_argument(
                "Unexpected payload, it should NOT be empty".to_string(),
            ));
        }

        let state: Arc<Mutex<ProviderState>> = self.provider_state.clone();

        // Define a retry strategy.
        let retry_strategy = ExponentialBackoff::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)
            .map(jitter) // add jitter to delays
            .take(Self::MAX_RETRIES);

        // Asynchronously perform the step.
        tokio::spawn(async move {
            let instance_value_json_str: String = {
                let instance_data: InstanceData = {
                    let lock: MutexGuard<ProviderState> = state.lock();
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

            let instance_json: serde_json::Value =
                serde_json::from_str(&instance_value_json_str).unwrap();

            let mut response_payload: String = "".to_string();

            let mut supported_method: bool = false;

            if (instance_json["@type"] == sdv::premium_airbag_seat_massager::ID
                || instance_json["@type"] == sdv::basic_airbag_seat_massager::ID)
                && targeted_payload.member_path == sdv::airbag_seat_massager::perform_step::NAME
            {
                supported_method = true;

                let response: sdv::airbag_seat_massager::perform_step::response::TYPE =
                    sdv::airbag_seat_massager::perform_step::response::TYPE {
                        status: sdv::airbag_seat_massager::status::TYPE {
                            code: 200,
                            message: "The step was performed successfully".to_string(),
                        },
                        ..Default::default()
                    };
                response_payload = serde_json::to_string(&response).unwrap();

                info!(
                    "Executed the operation {} on instance {}",
                    targeted_payload.member_path, targeted_payload.instance_id
                );
            }

            if !supported_method {
                return Err(format!(
                    "The instance with the instance id '{}' does not support the operation '{}'",
                    targeted_payload.instance_id, targeted_payload.operation
                ));
            }

            Retry::spawn(retry_strategy, || async {
                // Connect to the consumer.
                let mut client = RespondClient::connect(respond_uri.clone())
                    .await
                    .map_err(|err_msg| format!("Unable to connect due to: {err_msg}"))?;

                // Prepare the answer request.
                let answer_request = tonic::Request::new(AnswerRequest {
                    ask_id: ask_id.clone(),
                    payload: response_payload.clone(),
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
        let request_inner = request.into_inner();
        let respond_uri: String = request_inner.respond_uri.clone();
        let ask_id: String = request_inner.ask_id.clone();
        let payload: String = request_inner.payload.clone();

        info!("Received an ask request");
        info!("  respond_uri: {respond_uri}");
        info!("  ask_id: {ask_id}");

        // Deserialize the targeted payload.
        let targeted_payload_json: TargetedPayload = serde_json::from_str(&payload).unwrap();

        debug!("  instance_id: {}", targeted_payload_json.instance_id);
        debug!("  member_path: {}", targeted_payload_json.member_path);
        debug!("  operation: {}", targeted_payload_json.operation);

        if targeted_payload_json.operation == digital_twin_operation::GET {
            self.get(respond_uri, ask_id, targeted_payload_json).await
        } else if targeted_payload_json.operation == digital_twin_operation::INVOKE {
            self.invoke(respond_uri, ask_id, targeted_payload_json).await
        } else {
            Err(tonic::Status::invalid_argument(format!(
                "Unexpected operation '{}'",
                targeted_payload_json.operation
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
