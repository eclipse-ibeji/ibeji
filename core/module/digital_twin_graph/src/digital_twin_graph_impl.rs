// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::async_rpc::v1::request::{
    request_client::RequestClient, AskRequest,
};
use core_protobuf_data_access::async_rpc::v1::respond::AnswerRequest;
use core_protobuf_data_access::module::digital_twin_graph::v1::{
    digital_twin_graph_server::DigitalTwinGraph, FindRequest, FindResponse, GetRequest,
    GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
};
use core_protobuf_data_access::module::digital_twin_registry::v1::digital_twin_registry_client::DigitalTwinRegistryClient;
use core_protobuf_data_access::module::digital_twin_registry::v1::{
    EndpointInfo, FindByInstanceIdRequest, FindByInstanceIdResponse, FindByModelIdRequest,
    FindByModelIdResponse,
};
use log::{debug, warn};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout, Duration};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;
use uuid::Uuid;

use crate::{digital_twin_operation, digital_twin_protocol, TargetedPayload};

#[derive(Debug)]
pub struct DigitalTwinGraphImpl {
    /// Digital Twin Registry URI.
    digital_twin_registry_uri: String,
    /// Respond URI.
    respond_uri: String,
    /// The sender for the asynchronous channel for AnswerRequests.
    tx: Arc<broadcast::Sender<AnswerRequest>>,
}

impl DigitalTwinGraphImpl {
    /// The base duration in milliseconds for the backoff strategy.
    const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;

    /// The maximum number of retries for the backoff strategy.
    const MAX_RETRIES: usize = 100;

    /// The timeout period in milliseconds for the backoff strategy.
    const TIMEOUT_PERIOD_IN_MILLIS: u64 = 5000;

    /// Create a new instance of a DigitalTwinGraphImpl.
    ///
    /// # Arguments
    /// * `digital_twin_registry_uri` - The uri for the digital twin registry service.
    /// * `respond_uri` - The uri for the respond service.
    /// * `tx` - The sender for the asynchronous channel for AnswerRequest's.
    pub fn new(
        digital_twin_registry_uri: &str,
        respond_uri: &str,
        tx: Arc<broadcast::Sender<AnswerRequest>>,
    ) -> DigitalTwinGraphImpl {
        DigitalTwinGraphImpl {
            digital_twin_registry_uri: digital_twin_registry_uri.to_string(),
            respond_uri: respond_uri.to_string(),
            tx,
        }
    }

    /// Is the provided subset a subset of the provided superset?
    ///
    /// # Arguments
    /// * `subset` - The provided subset.
    /// * `superset` - The provided superset.
    fn is_subset(subset: &[String], superset: &[String]) -> bool {
        subset.iter().all(|subset_member| {
            superset.iter().any(|supserset_member| subset_member == supserset_member)
        })
    }

    /// Use the Digital Twin Registery service to find the endpoints for digital twin providers that support
    /// the specified model id, protocol and operations.
    ///
    /// # Arguments
    /// * `model_id` - The matching model id.
    /// * `protocol` - The required protocol.
    /// * `operations` - The required operations.
    pub async fn find_digital_twin_providers_with_model_id(
        &self,
        model_id: &str,
        protocol: &str,
        operations: &[String],
    ) -> Result<Vec<EndpointInfo>, String> {
        // Define the retry strategy.
        let retry_strategy = ExponentialBackoff::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)
            .map(jitter) // add jitter to delays
            .take(Self::MAX_RETRIES);

        let response: FindByModelIdResponse = Retry::spawn(retry_strategy.clone(), || async {
            let mut client =
                DigitalTwinRegistryClient::connect(self.digital_twin_registry_uri.to_string())
                    .await
                    .map_err(|error| format!("{error}"))?;

            let request =
                tonic::Request::new(FindByModelIdRequest { model_id: model_id.to_string() });

            client.find_by_model_id(request).await.map_err(|error| error.to_string())
        })
        .await?
        .into_inner();

        Ok(response
            .entity_access_info_list
            .iter()
            .flat_map(|entity_access_info| entity_access_info.endpoint_info_list.clone())
            .filter(|endpoint_info| {
                endpoint_info.protocol == protocol
                    && Self::is_subset(operations, &endpoint_info.operations)
            })
            .collect())
    }

    /// Use the Digital Twin Registry service to find the endpoints for digital twin providers that support the specified instance id, protocol and operations.
    ///
    /// # Arguments
    /// * `instance_id` - The matching instance id.
    /// * `protocol` - The required protocol.
    /// * `operations` - The required operations.
    pub async fn find_digital_twin_providers_with_instance_id(
        &self,
        instance_id: &str,
        protocol: &str,
        operations: &[String],
    ) -> Result<Vec<EndpointInfo>, String> {
        // Define the retry strategy.
        let retry_strategy = ExponentialBackoff::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)
            .map(jitter) // add jitter to delays
            .take(Self::MAX_RETRIES);

        let response: FindByInstanceIdResponse = Retry::spawn(retry_strategy.clone(), || async {
            let mut client =
                DigitalTwinRegistryClient::connect(self.digital_twin_registry_uri.to_string())
                    .await
                    .map_err(|error| format!("{error}"))?;

            let request = tonic::Request::new(FindByInstanceIdRequest {
                instance_id: instance_id.to_string(),
            });

            client.find_by_instance_id(request).await.map_err(|error| error.to_string())
        })
        .await?
        .into_inner();

        Ok(response
            .entity_access_info_list
            .iter()
            .flat_map(|entity_access_info| entity_access_info.endpoint_info_list.clone())
            .filter(|endpoint_info| {
                endpoint_info.protocol == protocol
                    && Self::is_subset(operations, &endpoint_info.operations)
            })
            .collect())
    }
}

#[tonic::async_trait]
impl DigitalTwinGraph for DigitalTwinGraphImpl {
    /// Find implementation.
    ///
    /// # Arguments
    /// * `request` - Find request.
    async fn find(
        &self,
        request: tonic::Request<FindRequest>,
    ) -> Result<tonic::Response<FindResponse>, tonic::Status> {
        let find_request = request.into_inner();
        let model_id = find_request.model_id;

        debug!("Received a find request for model id {model_id}");

        // Retrieve the provider details.
        let provider_endpoint_info_list = self
            .find_digital_twin_providers_with_model_id(
                model_id.as_str(),
                digital_twin_protocol::GRPC,
                &[digital_twin_operation::GET.to_string()],
            )
            .await
            .map_err(tonic::Status::internal)?;

        // Build a map of instance id to its associated endpoint infos.
        let instance_provider_map: std::collections::HashMap<String, Vec<EndpointInfo>> =
            provider_endpoint_info_list
                .iter()
                .map(|provider_endpoint_info| {
                    (provider_endpoint_info.context.clone(), provider_endpoint_info.clone())
                })
                .fold(
                    std::collections::HashMap::new(),
                    |mut accumulator, (instance_id, endpoint_info)| {
                        accumulator.entry(instance_id).or_insert_with(Vec::new).push(endpoint_info);
                        accumulator
                    },
                );

        let mut values = vec![];

        for instance_id in instance_provider_map.keys() {
            // We will only use the first provider. For a high availability scenario, we can try multiple providers.
            let provider_endpoint_info = &instance_provider_map[instance_id][0];

            let provider_uri = provider_endpoint_info.uri.clone();
            let instance_id = provider_endpoint_info.context.clone();

            let tx = Arc::clone(&self.tx);
            let mut rx = tx.subscribe();

            let client_result = RequestClient::connect(provider_uri.clone()).await;
            if client_result.is_err() {
                warn!("Unable to connect. We will skip this one.");
                continue;
            }
            let mut client = client_result.unwrap();

            // Note: The ask id must be a universally unique value.
            let ask_id = Uuid::new_v4().to_string();

            let targeted_payload = TargetedPayload {
                instance_id: instance_id.clone(),
                member_path: "".to_string(),
                operation: digital_twin_operation::GET.to_string(),
                payload: "".to_string(),
            };

            // Serialize the targeted payload.
            let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

            let request = tonic::Request::new(AskRequest {
                respond_uri: self.respond_uri.clone(),
                ask_id: ask_id.clone(),
                payload: targeted_payload_json.clone(),
            });

            // Send the ask.
            let response = client.ask(request).await;
            if let Err(status) = response {
                warn!("Unable to call ask, due to {status:?}\nWe will skip this one.");
                continue;
            }

            // Wait for the answer request.
            let mut answer_request: AnswerRequest = Default::default();
            let mut attempts_after_failure = 0;
            while attempts_after_failure < Self::MAX_RETRIES {
                match timeout(Duration::from_millis(Self::TIMEOUT_PERIOD_IN_MILLIS), rx.recv())
                    .await
                {
                    Ok(Ok(request)) => {
                        if ask_id == request.ask_id {
                            // We have received the answer request that we are expecting.
                            answer_request = request;
                            break;
                        } else {
                            // Ignore this answer request, as it is not the one that we are expecting.
                            // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                            continue;
                        }
                    }
                    Ok(Err(error_message)) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                        sleep(Duration::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                    Err(error_message) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                        sleep(Duration::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                }
            }

            debug!(
                "Received an answer request.  The ask_id is '{}'. The payload is '{}'",
                answer_request.ask_id, answer_request.payload
            );

            values.push(answer_request.payload);
        }

        debug!("Completed the find request");

        Ok(tonic::Response::new(FindResponse { values }))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        let get_request = request.into_inner();
        let instance_id = get_request.instance_id;
        let member_path = get_request.member_path;

        debug!("Received a get request for instance id {instance_id}");

        // Retrieve the provider details.
        let provider_endpoint_info_list = self
            .find_digital_twin_providers_with_instance_id(
                instance_id.as_str(),
                digital_twin_protocol::GRPC,
                &[digital_twin_operation::GET.to_string()],
            )
            .await
            .map_err(tonic::Status::internal)?;

        if provider_endpoint_info_list.is_empty() {
            return Err(tonic::Status::not_found("No providers found"));
        }

        // We will only use the first provider.
        let provider_endpoint_info = &provider_endpoint_info_list[0];

        let provider_uri = provider_endpoint_info.uri.clone();
        let instance_id = provider_endpoint_info.context.clone();

        let tx = Arc::clone(&self.tx);
        let mut rx = tx.subscribe();

        let client_result = RequestClient::connect(provider_uri.clone()).await;
        if client_result.is_err() {
            return Err(tonic::Status::internal("Unable to connect to the provider."));
        }
        let mut client = client_result.unwrap();

        // Note: The ask id must be a universally unique value.
        let ask_id = Uuid::new_v4().to_string();

        let targeted_payload = TargetedPayload {
            instance_id: instance_id.clone(),
            member_path: member_path.clone(),
            operation: digital_twin_operation::GET.to_string(),
            payload: "".to_string(),
        };

        // Serialize the targeted payload.
        let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

        let request = tonic::Request::new(AskRequest {
            respond_uri: self.respond_uri.clone(),
            ask_id: ask_id.clone(),
            payload: targeted_payload_json.clone(),
        });

        // Send the ask.
        let response = client.ask(request).await;
        if let Err(status) = response {
            return Err(tonic::Status::internal(format!("Unable to call ask, due to {status:?}")));
        }

        // Wait for the answer request.
        let mut answer_request: AnswerRequest = Default::default();
        let mut attempts_after_failure = 0;
        while attempts_after_failure < Self::MAX_RETRIES {
            match timeout(Duration::from_millis(Self::TIMEOUT_PERIOD_IN_MILLIS), rx.recv()).await {
                Ok(Ok(request)) => {
                    if ask_id == request.ask_id {
                        // We have received the answer request that we are expecting.
                        answer_request = request;
                        break;
                    } else {
                        // Ignore this answer request, as it is not the one that we are expecting.
                        // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                        continue;
                    }
                }
                Ok(Err(error_message)) => {
                    warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                    sleep(Duration::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)).await;
                    attempts_after_failure += 1;
                    continue;
                }
                Err(error_message) => {
                    warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                    sleep(Duration::from_millis(Self::BACKOFF_BASE_DURATION_IN_MILLIS)).await;
                    attempts_after_failure += 1;
                    continue;
                }
            }
        }

        debug!(
            "Received an answer request.  The ask_id is '{}'. The payload is '{}",
            answer_request.ask_id, answer_request.payload
        );

        Ok(tonic::Response::new(GetResponse { value: answer_request.payload.clone() }))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(
        &self,
        request: tonic::Request<SetRequest>,
    ) -> Result<tonic::Response<SetResponse>, tonic::Status> {
        warn!("Got a set request: {request:?}");

        Err(tonic::Status::unimplemented("set has not been implemented"))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(
        &self,
        request: tonic::Request<InvokeRequest>,
    ) -> Result<tonic::Response<InvokeResponse>, tonic::Status> {
        let invoke_request = request.into_inner();
        let instance_id = invoke_request.instance_id;
        let member_path = invoke_request.member_path;
        let request_payload = invoke_request.request_payload;

        debug!("Received an invoke request for instance id {instance_id}");

        // Retrieve the provider details.
        let provider_endpoint_info_list = self
            .find_digital_twin_providers_with_instance_id(
                instance_id.as_str(),
                digital_twin_protocol::GRPC,
                &[digital_twin_operation::INVOKE.to_string()],
            )
            .await
            .map_err(tonic::Status::internal)?;

        if provider_endpoint_info_list.is_empty() {
            return Err(tonic::Status::not_found("No providers found"));
        }

        // We will only use the first provider.
        let provider_endpoint_info = &provider_endpoint_info_list[0];

        let provider_uri = provider_endpoint_info.uri.clone();
        let instance_id = provider_endpoint_info.context.clone();

        let tx = Arc::clone(&self.tx);
        let mut rx = tx.subscribe();

        let client_result = RequestClient::connect(provider_uri.clone()).await;
        if client_result.is_err() {
            return Err(tonic::Status::internal("Unable to connect to the provider."));
        }
        let mut client = client_result.unwrap();

        // Note: The ask id must be a universally unique value.
        let ask_id = Uuid::new_v4().to_string();

        let targeted_payload = TargetedPayload {
            instance_id: instance_id.clone(),
            member_path: member_path.clone(),
            operation: digital_twin_operation::INVOKE.to_string(),
            payload: request_payload.to_string(),
        };

        // Serialize the targeted payload.
        let targeted_payload_json = serde_json::to_string_pretty(&targeted_payload).unwrap();

        let request = tonic::Request::new(AskRequest {
            respond_uri: self.respond_uri.clone(),
            ask_id: ask_id.clone(),
            payload: targeted_payload_json.clone(),
        });

        // Send the ask.
        let response = client.ask(request).await;
        if let Err(status) = response {
            return Err(tonic::Status::internal(format!("Unable to call ask, due to {status:?}")));
        }

        // Wait for the answer request.
        let mut answer_request: AnswerRequest = Default::default();
        let mut attempts_after_failure = 0;
        const MAX_ATTEMPTS_AFTER_FAILURE: u8 = 10;
        while attempts_after_failure < MAX_ATTEMPTS_AFTER_FAILURE {
            match timeout(Duration::from_millis(Self::TIMEOUT_PERIOD_IN_MILLIS), rx.recv()).await {
                Ok(Ok(request)) => {
                    if ask_id == request.ask_id {
                        // We have received the answer request that we are expecting.
                        answer_request = request;
                        break;
                    } else {
                        // Ignore this answer request, as it is not the one that we are expecting.
                        // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                        continue;
                    }
                }
                Ok(Err(error_message)) => {
                    warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                    sleep(Duration::from_secs(1)).await;
                    attempts_after_failure += 1;
                    continue;
                }
                Err(error_message) => {
                    warn!("Failed to receive the answer request.  The error message is '{}'.  We may retry in a moment.", error_message);
                    sleep(Duration::from_secs(1)).await;
                    attempts_after_failure += 1;
                    continue;
                }
            }
        }

        debug!(
            "Received an answer request.  The ask_id is '{}'. The payload is '{}",
            answer_request.ask_id, answer_request.payload
        );

        Ok(tonic::Response::new(InvokeResponse {
            response_payload: answer_request.payload.clone(),
        }))
    }
}
