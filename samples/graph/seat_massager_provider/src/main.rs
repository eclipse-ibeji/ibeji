// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod request_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use parking_lot::Mutex;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::provider_config;
use samples_common::utils::retrieve_invehicle_digital_twin_uri;
use samples_protobuf_data_access::async_rpc::v1::request::request_server::RequestServer;
use samples_protobuf_data_access::digital_twin_registry::v1::digital_twin_registry::digital_twin_registry_client::DigitalTwinRegistryClient;
use samples_protobuf_data_access::digital_twin_registry::v1::digital_twin_registry::{
    EndpointInfo, EntityAccessInfo, RegisterRequest, RegisterResponse,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{transport::Server, Status};
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

use crate::request_impl::{InstanceData, ProviderState, RequestImpl};

const BACKOFF_BASE_DURATION_IN_MILLIS: u64 = 100;
const MAX_RETRIES: usize = 100;

/// Add an entry to the instance map.
/// # Arguments
/// * `instance_map` - The instance map.
/// * `instance_id` - The instance id.
/// * `model_id` - The model id.
/// * `description` - The description.
/// * `serialized_value` - The serialized value.
fn add_entry_to_instance_map(
    instance_map: &mut HashMap<String, InstanceData>,
    instance_id: String,
    model_id: String,
    description: String,
    serialized_value: String,
) {
    instance_map.insert(instance_id, InstanceData { model_id, description, serialized_value });
}

/// Create the provider's state.
fn create_provider_state() -> ProviderState {
    let mut result: ProviderState = ProviderState { instance_map: HashMap::new() };

    // Create the seat massagers.

    let front_left_airbag_seat_massager_instance_id = "front_left_airbag_seat_massager".to_string();
    let front_left_airbag_seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        sdv::premium_airbag_seat_massager::TYPE {
            instance_id: front_left_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };

    let front_right_airbag_seat_massager_instance_id =
        "front_right_airbag_seat_massager".to_string();
    let front_right_airbag_seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        sdv::premium_airbag_seat_massager::TYPE {
            instance_id: front_right_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };

    let back_left_airbag_seat_massager_instance_id = "back_left_airbag_seat_massager".to_string();
    let back_left_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_left_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };

    let back_center_airbag_seat_massager_instance_id =
        "back_center_airbag_seat_massager".to_string();
    let back_center_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_center_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };

    let back_right_airbag_seat_massager_instance_id = "back_right_airbag_seat_massager".to_string();
    let back_right_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_right_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };

    // Build the instance map.

    add_entry_to_instance_map(
        &mut result.instance_map,
        front_left_airbag_seat_massager_instance_id.clone(),
        sdv::premium_airbag_seat_massager::ID.to_string(),
        sdv::premium_airbag_seat_massager::DESCRIPTION.to_string(),
        serde_json::to_string(&front_left_airbag_seat_massager).unwrap(),
    );

    add_entry_to_instance_map(
        &mut result.instance_map,
        front_right_airbag_seat_massager_instance_id.clone(),
        sdv::premium_airbag_seat_massager::ID.to_string(),
        sdv::premium_airbag_seat_massager::DESCRIPTION.to_string(),
        serde_json::to_string(&front_right_airbag_seat_massager).unwrap(),
    );

    add_entry_to_instance_map(
        &mut result.instance_map,
        back_left_airbag_seat_massager_instance_id.clone(),
        sdv::basic_airbag_seat_massager::ID.to_string(),
        sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
        serde_json::to_string(&back_left_airbag_seat_massager).unwrap(),
    );

    add_entry_to_instance_map(
        &mut result.instance_map,
        back_center_airbag_seat_massager_instance_id.clone(),
        sdv::basic_airbag_seat_massager::ID.to_string(),
        sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
        serde_json::to_string(&back_center_airbag_seat_massager).unwrap(),
    );

    add_entry_to_instance_map(
        &mut result.instance_map,
        back_right_airbag_seat_massager_instance_id.clone(),
        sdv::basic_airbag_seat_massager::ID.to_string(),
        sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
        serde_json::to_string(&back_right_airbag_seat_massager).unwrap(),
    );

    result
}

/// Register the airbag seat massagers.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
/// * `provider_state` - The provider's state.
async fn register_seat_massagers(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
    provider_state: Arc<Mutex<ProviderState>>,
) -> Result<(), Status> {
    let mut entity_access_info_list: Vec<EntityAccessInfo> = Vec::new();

    provider_state.lock().instance_map.iter().for_each(|(instance_id, instance_data)| {
        info!(
            "Registering the instance with the instance id '{}' and the model id '{}'",
            instance_id, instance_data.model_id
        );

        let endpoint_info = EndpointInfo {
            protocol: digital_twin_protocol::GRPC.to_string(),
            operations: vec![
                digital_twin_operation::GET.to_string(),
                digital_twin_operation::INVOKE.to_string(),
            ],
            uri: provider_uri.to_string(),
            context: instance_id.to_string(),
        };

        let entity_access_info = EntityAccessInfo {
            name: String::new(), // no name, so we will use an empty name
            id: instance_data.model_id.to_string(),
            description: instance_data.description.to_string(),
            endpoint_info_list: vec![endpoint_info],
        };

        entity_access_info_list.push(entity_access_info);
    });

    let retry_strategy = ExponentialBackoff::from_millis(BACKOFF_BASE_DURATION_IN_MILLIS)
        .map(jitter) // add jitter to delays
        .take(MAX_RETRIES);

    let result: Result<RegisterResponse, Status> = Retry::spawn(retry_strategy.clone(), || async {
        let mut client = DigitalTwinRegistryClient::connect(invehicle_digital_twin_uri.to_string())
            .await
            .map_err(|e: tonic::transport::Error| Status::internal(e.to_string()))?;

        let request = tonic::Request::new(RegisterRequest {
            entity_access_info_list: entity_access_info_list.clone(),
        });

        info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");

        let response: RegisterResponse = client
            .register(request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_inner();
        Ok(response)
    })
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(status) => Err(status),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Seat Massager Provider has started.");

    let settings =
        provider_config::load_settings_with_config_filename("seat_massager_provider_settings");

    let provider_authority = settings.provider_authority;

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await?;

    // Construct the provider URI from the provider authority.
    let provider_uri = format!("http://{provider_authority}"); // Devskim: ignore DS137138

    // Setup the HTTP server.
    let addr: SocketAddr = provider_authority.parse()?;
    let state = Arc::new(Mutex::new(create_provider_state()));
    let request_impl = RequestImpl { provider_state: state.clone() };
    let server_future = Server::builder().add_service(RequestServer::new(request_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    register_seat_massagers(&invehicle_digital_twin_uri, &provider_uri, state.clone()).await?;

    server_future.await?;

    Ok(())
}
