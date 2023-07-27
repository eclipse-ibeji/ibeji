// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::chariott::service_discovery::core::v1::service_registry_client::ServiceRegistryClient;
use core_protobuf_data_access::chariott::service_discovery::core::v1::{RegisterRequest, ServiceMetadata};
use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
use env_logger::{Builder, Target};
use log::{debug, error, info, LevelFilter};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Status};

mod invehicle_digital_twin_impl;
mod invehicle_digital_twin_config;

pub const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "invehicle_digital_twin";
pub const INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
pub const CHARIOTT_NAMESPACE_FOR_IBEJI: &str = "sdv.ibeji";
pub const CHARIOTT_COMMUNICATION_KIND_FOR_GRPC: &str = "grpc+proto";
pub const CHARIOTT_COMMUNICATION_REFERENCE_FOR_INVEHICLE_DIGITAL_TWIN_SERVICE: &str = "invehicle_digital_twin.v1";

/// Register the invehicle digital twin service with Chariott.
///
/// # Arguments
/// * `chariott_uri` - Chariott's URI.
/// * `invehicle_digital_twin_uri` - In-vehicle Digital Twin Service's URI.
pub async fn register_invehicle_digital_twin_service_with_chariott(
    chariott_uri: &str,
    invehicle_digital_twin_uri: &str,
) -> Result<(), Status> {
    let mut client = ServiceRegistryClient::connect(chariott_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let service = Some(ServiceMetadata {
        namespace: CHARIOTT_NAMESPACE_FOR_IBEJI.to_string(),
        name: INVEHICLE_DIGITAL_TWIN_SERVICE_NAME.to_string(),
        version: INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION.to_string(),
        uri: invehicle_digital_twin_uri.to_string(),
        communication_kind: CHARIOTT_COMMUNICATION_KIND_FOR_GRPC.to_string(),
        communication_reference: CHARIOTT_COMMUNICATION_REFERENCE_FOR_INVEHICLE_DIGITAL_TWIN_SERVICE.to_string(),
    });

    let request = Request::new(RegisterRequest { service });

    let _response = client
        .register(request)
        .await
        .map_err(|_| Status::internal("Chariott register request failed"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Load the config.
    let settings = invehicle_digital_twin_config::load_settings();
    let invehicle_digital_twin_authority = settings.invehicle_digital_twin_authority;
    let chariott_uri_option = settings.chariott_uri;

    // Setup the HTTP server.
    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;
    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    let server_future = Server::builder()
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr);
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");

    // Register the digital twin service with Chariott if Chariott's URI was provided in the config.
    // Note: We are not using Chariott's announce, and therefore the digital twin serice will be forcibly unregistered
    //       after 15 seconds unless the CHARIOTT_REGISTRY_TTL_SECS environment variable is set. Please make sure that
    //       it is set (and exported) in the shell running Chariott before Chariott has started.
    if chariott_uri_option.is_some() {
        let response = register_invehicle_digital_twin_service_with_chariott(
            &chariott_uri_option.unwrap(),
            &invehicle_digital_twin_address,
        )
        .await;
        if let Err(error) = response {
            error!("Failed to register this service with Chariott: '{error}'");
            return Err(error)?;
        }
        info!("This service is now registered with Chariott.");
    } else {
        info!("This service is not using Chariott.");
    }

    server_future.await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
