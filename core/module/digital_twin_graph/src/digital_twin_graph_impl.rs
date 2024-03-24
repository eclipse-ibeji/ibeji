// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::{debug, info, warn};
use core_protobuf_data_access::async_rpc::v1::request::{request_client::RequestClient, AskRequest};
use core_protobuf_data_access::async_rpc::v1::respond::AnswerRequest;
use core_protobuf_data_access::module::digital_twin_registry::v1::digital_twin_registry_client::DigitalTwinRegistryClient;
use core_protobuf_data_access::module::digital_twin_registry::v1::{EndpointInfo, FindByModelIdRequest, FindByInstanceIdRequest};
use core_protobuf_data_access::module::digital_twin_graph::v1::{
    digital_twin_graph_server::DigitalTwinGraph, FindRequest, FindResponse, GetRequest, GetResponse, SetRequest, SetResponse, InvokeRequest, InvokeResponse,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout, Duration};
use uuid::Uuid;

use crate::{digital_twin_operation, digital_twin_protocol, TargetedPayload};

#[derive(Debug)]
pub struct DigitalTwinGraphImpl {
    invehicle_digital_twin_uri: String,    
    respond_uri: String,
    tx: Arc<broadcast::Sender<AnswerRequest>>,
}

impl DigitalTwinGraphImpl {
    /// Create a new instance of a DigitalTwinGraphImpl.
    ///
    /// # Arguments
    /// * `invehicle_digital_twin_uri` - The uri for the invehicle digital twin service.
    /// * `respond_uri` - The uri for the respond service.
    /// * `tx` - The sender for the asynchronous channel for AnswerRequest's.
    pub fn new(
        invehicle_digital_twin_uri: &str,        
        respond_uri: &str,
        tx: Arc<broadcast::Sender<AnswerRequest>>,
    ) -> DigitalTwinGraphImpl {
        DigitalTwinGraphImpl {
            invehicle_digital_twin_uri: invehicle_digital_twin_uri.to_string(),            
            respond_uri: respond_uri.to_string(),
            tx: tx,
        }
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

/// Use Ibeji to discover the endpoints for digital twin providers that satisfy the requirements.
///
/// # Arguments
/// * `digitial_twin_registry_service_uri` - Digital Twin Registry Service URI.
/// * `model_id` - The matching model id.
/// * `protocol` - The required protocol.
/// * `operations` - The required operations.
pub async fn discover_digital_twin_providers_with_model_id(
    digitial_twin_registry_service_uri: &str,
    model_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<Vec::<EndpointInfo>, String> {
    info!("Sending a find_by_model_id request for model id {model_id} to the Digital Twin Registry Service at {digitial_twin_registry_service_uri}");

    let mut client =
        DigitalTwinRegistryClient::connect(digitial_twin_registry_service_uri.to_string())
            .await
            .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByModelIdRequest { model_id: model_id.to_string() });
    let response = client.find_by_model_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_model_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info_list);

    
    // Ok(response_inner.entity_access_info_list.iter().map(|entity_access_info| entity_access_info.endpoint_info_list.clone()).flatten().collect())

    Ok(response_inner.entity_access_info_list.iter()
        .map(|entity_access_info| entity_access_info.endpoint_info_list.clone())
        .flatten()
        .filter(|endpoint_info| {
            endpoint_info.protocol == protocol
                && is_subset(operations, &endpoint_info.operations)
        })
        .collect())
}


/// Use Ibeji to discover the endpoints for digital twin providers that satisfy the requirements.
///
/// # Arguments
/// * `digitial_twin_registry_service_uri` - Digital Twin Registry Service URI.
/// * `instance_id` - The matching instance id.
/// * `protocol` - The required protocol.
/// * `operations` - The required operations.
pub async fn discover_digital_twin_providers_with_instance_id(
    digitial_twin_registry_service_uri: &str,
    instance_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<Vec::<EndpointInfo>, String> {
    info!("Sending a find_by_instance_id request for instance id {instance_id} to the Digital Twin Registry Service at {digitial_twin_registry_service_uri}");

    let mut client =
        DigitalTwinRegistryClient::connect(digitial_twin_registry_service_uri.to_string())
            .await
            .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByInstanceIdRequest { instance_id: instance_id.to_string() });
    let response = client.find_by_instance_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_instance_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info_list);

    Ok(response_inner.entity_access_info_list.iter()
        .map(|entity_access_info| entity_access_info.endpoint_info_list.clone())
        .flatten()
        .filter(|endpoint_info| {
            endpoint_info.protocol == protocol
                && is_subset(operations, &endpoint_info.operations)
        })
        .collect())
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
        let request_inner = request.into_inner();
        let model_id = request_inner.model_id;

        info!("Received a find request for model id {model_id}");

        // Retrieve the provider details.
        let provider_endpoint_info_list = discover_digital_twin_providers_with_model_id(
            &self.invehicle_digital_twin_uri,
            model_id.as_str(),
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::GET.to_string()],
        )
        .await.map_err(|error| tonic::Status::internal(error))?;

        info!(">> Found the provider endpoint info list: {provider_endpoint_info_list:?}");

        let mut values = vec![];

        for provider_endpoint_info in &provider_endpoint_info_list {
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
                warn!("Unable to call ask, due to {status:?}\nWe will skip this one..");
                continue;
            }

            // Wait for the answer request.
            let mut answer_request: AnswerRequest = Default::default();
            let mut attempts_after_failure = 0;
            const MAX_ATTEMPTS_AFTER_FAILURE: u8 = 10;
            while attempts_after_failure < MAX_ATTEMPTS_AFTER_FAILURE {
                match timeout(Duration::from_secs(5), rx.recv()).await {
                    Ok(Ok(request)) => {
                        if ask_id == request.ask_id {
                            // We have received the answer request that we are expecting.
                            answer_request = request;
                            break;
                        } else {
                            // Ignore this answer request, as it is not the one that we are expecting.
                            warn!("Received an unexpected answer request with ask_id '{}'.  We will retry in a moment.", request.ask_id);
                            // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                            continue;
                        }
                    }
                    Ok(Err(error_message)) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We will retry in a moment.", error_message);
                        sleep(Duration::from_secs(1)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                    Err(error_message) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We will retry in a moment.", error_message);
                        sleep(Duration::from_secs(1)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                }
            }

            info!(
                "Received an answer request.  The ask_id is '{}'. The payload is '{}",
                answer_request.ask_id, answer_request.payload
            );

            values.push(answer_request.payload);
        }

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
        let request_inner = request.into_inner();
        let instance_id = request_inner.instance_id;

        info!("Received a get request for instance id {instance_id}");

        // Retrieve the provider details.
        let provider_endpoint_info_list = discover_digital_twin_providers_with_instance_id(
            &self.invehicle_digital_twin_uri,
            instance_id.as_str(),
            digital_twin_protocol::GRPC,
            &[digital_twin_operation::GET.to_string()],
        )
        .await.map_err(|error| tonic::Status::internal(error))?;

        info!(">> Found the provider endpoint info list: {provider_endpoint_info_list:?}");

        let mut values = vec![];

        for provider_endpoint_info in &provider_endpoint_info_list {
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
                warn!("Unable to call ask, due to {status:?}\nWe will skip this one..");
                continue;
            }

            // Wait for the answer request.
            let mut answer_request: AnswerRequest = Default::default();
            let mut attempts_after_failure = 0;
            const MAX_ATTEMPTS_AFTER_FAILURE: u8 = 10;
            while attempts_after_failure < MAX_ATTEMPTS_AFTER_FAILURE {
                match timeout(Duration::from_secs(5), rx.recv()).await {
                    Ok(Ok(request)) => {
                        if ask_id == request.ask_id {
                            // We have received the answer request that we are expecting.
                            answer_request = request;
                            break;
                        } else {
                            // Ignore this answer request, as it is not the one that we are expecting.
                            warn!("Received an unexpected answer request with ask_id '{}'.  We will retry in a moment.", request.ask_id);
                            // Immediately try again.  This was not a failure, so we do not increment attempts_after_failure or sleep.
                            continue;
                        }
                    }
                    Ok(Err(error_message)) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We will retry in a moment.", error_message);
                        sleep(Duration::from_secs(1)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                    Err(error_message) => {
                        warn!("Failed to receive the answer request.  The error message is '{}'.  We will retry in a moment.", error_message);
                        sleep(Duration::from_secs(1)).await;
                        attempts_after_failure += 1;
                        continue;
                    }
                }
            }

            info!(
                "Received an answer request.  The ask_id is '{}'. The payload is '{}",
                answer_request.ask_id, answer_request.payload
            );

            values.push(answer_request.payload);
        }

        if values.is_empty() {
            return Err(tonic::Status::not_found("No values found"));
        }

        Ok(tonic::Response::new(GetResponse { value: values[0].clone()}))
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
        warn!("Got a invoke request: {request:?}");

        Err(tonic::Status::unimplemented("invoke has not been implemented"))
    }
}
