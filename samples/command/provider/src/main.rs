// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod provider_impl;

use env_logger::{Builder, Target};
use ibeji_common::{find_full_path, retrieve_dtdl};
use log::{debug, info, LevelFilter};
use parking_lot::Mutex;
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::RegisterRequest;
use proto::provider::provider_server::ProviderServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

const IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI: &str = "http://[::1]:50010"; // Devskim: ignore DS137138
const PROVIDER_ADDR: &str = "[::1]:40010"; // Devskim: ignore DS137138

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    debug!("Preparing the Provider's DTDL.");
    let provider_dtdl_path = find_full_path("content/show_notification.json")?;
    let dtdl = retrieve_dtdl(&provider_dtdl_path)?;
    debug!("Prepared the Provider's DTDL.");

    // Setup the HTTP server.
    let addr: SocketAddr = PROVIDER_ADDR.parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(ProviderServer::new(provider_impl)).serve(addr);
    info!("The HTTP server is listening on address '{PROVIDER_ADDR}'");

    info!("Sending a register request with the Provider's DTDL to the In-Vehicle Digital Twin Service URI {IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI}");
    let mut client = DigitalTwinClient::connect(IN_VEHICLE_DIGITAL_TWIN_SERVICE_URI).await?;
    let request = tonic::Request::new(RegisterRequest { dtdl });
    let _response = client.register(request).await?;
    debug!("The Provider's DTDL has been registered.");

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
