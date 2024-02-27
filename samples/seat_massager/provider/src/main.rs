// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod request_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{retrieve_invehicle_digital_twin_uri, retry_async_based_on_status};
use samples_common::provider_config;
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::async_rpc::v1::request::request_server::RequestServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use tonic::{Status, transport::Server};

use crate::request_impl::{RequestImpl, RequestState};

/// Register the airbag seat massager's massage airbags property.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
async fn register_airbag_massager(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
) -> Result<(), Status> {
    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![
            digital_twin_operation::GET.to_string(),
            digital_twin_operation::INVOKE.to_string(),
        ],
        uri: provider_uri.to_string(),
        context: sdv::premium_airbag_seat_massager::ID.to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: String::new(),    // no name, so we will use an empty name
        id: sdv::premium_airbag_seat_massager::ID.to_string(),
        description: sdv::premium_airbag_seat_massager::DESCRIPTION.to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    let mut client = InvehicleDigitalTwinClient::connect(invehicle_digital_twin_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let settings = provider_config::load_settings();

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
    let state = Arc::new(Mutex::new(RequestState { }));
    let request_impl = RequestImpl { state };
    let server_future =
        Server::builder().add_service(RequestServer::new(request_impl)).serve(addr);
    info!("The HTTP server is listening on address '{provider_authority}'");

    info!("Sending a register request to the In-Vehicle Digital Twin Service URI {invehicle_digital_twin_uri}");
    retry_async_based_on_status(30, Duration::from_secs(1), || {
        register_airbag_massager(&invehicle_digital_twin_uri, &provider_uri)
    })
    .await?;

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
