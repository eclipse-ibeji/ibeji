// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod request_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::provider_config;
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_protobuf_data_access::async_rpc::v1::request::request_server::RequestServer;
use samples_protobuf_data_access::digital_twin_registry::v1::digital_twin_registry::digital_twin_registry_client::DigitalTwinRegistryClient;
use samples_protobuf_data_access::digital_twin_registry::v1::digital_twin_registry::{
    EndpointInfo, EntityAccessInfo, RegisterRequest,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use tonic::{transport::Server, Status};

use crate::request_impl::{InstanceData, RequestImpl, RequestState};

fn create_request_state() -> RequestState {
    let mut result: RequestState = RequestState { instance_map: HashMap::new() };

    // Create the seat massagers.

    let front_left_airbag_seat_massager_instance_id = "front_left_airbag_seat_massager".to_string();
    let front_left_airbag_seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        sdv::premium_airbag_seat_massager::TYPE {
            instance_id: front_left_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };
    result.instance_map.insert(
        front_left_airbag_seat_massager_instance_id.to_string(),
        InstanceData {
            model_id: sdv::premium_airbag_seat_massager::ID.to_string(),
            description: sdv::premium_airbag_seat_massager::DESCRIPTION.to_string(),
            serialized_value: serde_json::to_string(&front_left_airbag_seat_massager).unwrap(),
        },
    );

    let front_right_airbag_seat_massager_instance_id =
        "front_right_airbag_seat_massager".to_string();
    let front_right_airbag_seat_massager: sdv::premium_airbag_seat_massager::TYPE =
        sdv::premium_airbag_seat_massager::TYPE {
            instance_id: front_right_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };
    result.instance_map.insert(
        front_right_airbag_seat_massager_instance_id.to_string(),
        InstanceData {
            model_id: sdv::premium_airbag_seat_massager::ID.to_string(),
            description: sdv::premium_airbag_seat_massager::DESCRIPTION.to_string(),
            serialized_value: serde_json::to_string(&front_right_airbag_seat_massager).unwrap(),
        },
    );

    let back_left_airbag_seat_massager_instance_id = "back_left_airbag_seat_massager".to_string();
    let back_left_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_left_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };
    result.instance_map.insert(
        back_left_airbag_seat_massager_instance_id.to_string(),
        InstanceData {
            model_id: sdv::basic_airbag_seat_massager::ID.to_string(),
            description: sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
            serialized_value: serde_json::to_string(&back_left_airbag_seat_massager).unwrap(),
        },
    );

    let back_center_airbag_seat_massager_instance_id =
        "back_center_airbag_seat_massager".to_string();
    let back_center_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_center_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };
    result.instance_map.insert(
        back_center_airbag_seat_massager_instance_id.to_string(),
        InstanceData {
            model_id: sdv::basic_airbag_seat_massager::ID.to_string(),
            description: sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
            serialized_value: serde_json::to_string(&back_center_airbag_seat_massager).unwrap(),
        },
    );

    let back_right_airbag_seat_massager_instance_id = "back_right_airbag_seat_massager".to_string();
    let back_right_airbag_seat_massager: sdv::basic_airbag_seat_massager::TYPE =
        sdv::basic_airbag_seat_massager::TYPE {
            instance_id: back_right_airbag_seat_massager_instance_id.clone(),
            ..Default::default()
        };
    result.instance_map.insert(
        back_right_airbag_seat_massager_instance_id.to_string(),
        InstanceData {
            model_id: sdv::basic_airbag_seat_massager::ID.to_string(),
            description: sdv::basic_airbag_seat_massager::DESCRIPTION.to_string(),
            serialized_value: serde_json::to_string(&back_right_airbag_seat_massager).unwrap(),
        },
    );

    result
}

/// Register the airbag seat massager's massage airbags property.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
/// * `instance_id` - The instance id.
async fn register_seat_massagers(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
    state: Arc<Mutex<RequestState>>,
) -> Result<(), Status> {
    let mut entity_access_info_list: Vec<EntityAccessInfo> = Vec::new();

    state.lock().instance_map.iter().for_each(|(instance_id, instance_data)| {
        info!(
            "Registering the instance with the instance id '{}' and the model id '{}'",
            instance_id, instance_data.model_id
        );

        let endpoint_info = EndpointInfo {
            protocol: digital_twin_protocol::GRPC.to_string(),
            operations: vec![digital_twin_operation::GET.to_string()],
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

    let mut client = DigitalTwinRegistryClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request = tonic::Request::new(RegisterRequest { entity_access_info_list });
    let _response = client.register(request).await?;

    Ok(())
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
    let state = Arc::new(Mutex::new(create_request_state()));
    let request_impl = RequestImpl { state: state.clone() };
    let server_future = Server::builder().add_service(RequestServer::new(request_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_seat_massagers(&invehicle_digital_twin_uri, &provider_uri, state.clone())
    })
    .await?;

    server_future.await?;

    debug!("The Seat Massager Provider has completed.");

    Ok(())
}
