// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use samples_common::{digital_twin_operation, digital_twin_protocol};
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, EntityAccessInfo, RegisterRequest};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const PROVIDER_AUTHORITY: &str = "[::1]:40010";

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let endpoint_info = EndpointInfo {
        protocol: digital_twin_protocol::GRPC.to_string(),
        operations: vec![digital_twin_operation::INVOKE.to_string()],
        uri: "http://[::1]:40010".to_string(), // Devskim: ignore DS137138
        context: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
    };

    let entity_access_info = EntityAccessInfo {
        name: "ShowNotification".to_string(),
        id: sdv::vehicle::cabin::infotainment::hmi::show_notification::ID.to_string(),
        description: "Show a notification on the HMI.".to_string(),
        endpoint_info_list: vec![endpoint_info],
    };

    // Setup the HTTP server.
    let addr: SocketAddr = PROVIDER_AUTHORITY.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{PROVIDER_AUTHORITY}'");

    info!("Sending a register request with the Provider's DTDL to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request =
        tonic::Request::new(RegisterRequest { entity_access_info_list: vec![entity_access_info] });
    let _response = client.register(request).await?;
    debug!("The Provider's DTDL has been registered.");

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
