// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use data_exchange::digitaltwin::{Entity, Endpoint, RegisterRequestPayload};
use dt_model_identifiers::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::RegisterRequest;
use samples_proto::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const PROVIDER_ADDR: &str = "[::1]:40010";

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    let mut operations = Vec::new();
    operations.push(String::from("Get"));
    operations.push(String::from("Set"));

    let endpoint = Endpoint {
        protocol: String::from("grpc"),
        operations,
        uri: String::from("http://[::1]:40010"),
        context: String::from(sdv::vehicle::cabin::infotainment::hmi::show_notification::ID)
    };

    let mut endpoints = Vec::new();
    endpoints.push(endpoint);

    let entity = Entity {
        name: String::from("ShowNotification"),
        id: String::from(sdv::vehicle::cabin::infotainment::hmi::show_notification::ID),
        description: String::from("Show a notification on the HMI."),
        endpoints
    };

    let mut entities = Vec::new();
    entities.push(entity);

    // Setup the HTTP server.
    let addr: SocketAddr = PROVIDER_ADDR.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(DigitalTwinProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{PROVIDER_ADDR}'");

    info!("Sending a register request with the Provider's DTDL to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request_payload = RegisterRequestPayload {
        entities
    };
    let request_payload = match serde_json::to_string(&request_payload) {
        Ok(content) => content,
        Err(error) => panic!("Failed to serialize the request payload: {error}")
    };
    let request = tonic::Request::new(RegisterRequest {
        payload: request_payload
    });
    let _response = client.register(request).await?;
    debug!("The Provider's DTDL has been registered.");

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
