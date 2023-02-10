// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod provider_impl;

use env_logger::{Builder, Target};
use ibeji_common::{find_full_path, retrieve_dtdl};
use log::{debug, info, LevelFilter};
use proto::digitaltwin::digital_twin_client::DigitalTwinClient;
use proto::digitaltwin::RegisterRequest;
use proto::provider::provider_server::ProviderServer;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tonic::transport::Server;

use crate::provider_impl::{ProviderImpl, SubscriptionMap};

#[tokio::main]
#[allow(clippy::collapsible_else_if)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Provider has started.");

    debug!("Preparing the Provider's DTDL.");
    let provider_dtdl_path = find_full_path("content/send_notification.json")?;
    let dtdl = retrieve_dtdl(&provider_dtdl_path)?;
    debug!("Prepared the Provider's DTDL.");

    // Setup the HTTP server.
    let addr: SocketAddr = "[::1]:40010".parse()?;
    let subscription_map = Arc::new(Mutex::new(SubscriptionMap::new()));
    let provider_impl = ProviderImpl { subscription_map: subscription_map.clone() };
    let server_future =
        Server::builder().add_service(ProviderServer::new(provider_impl)).serve(addr);

    debug!("Registering the Provider's DTDL with the Digital Twin Service.");
    let mut client = DigitalTwinClient::connect("http://[::1]:50010").await?; // Devskim: ignore DS137138
    let request = tonic::Request::new(RegisterRequest { dtdl });
    let _response = client.register(request).await?;
    info!("The Provider's DTDL has been registered.");

    server_future.await?;

    debug!("The Provider has completed.");

    Ok(())
}
