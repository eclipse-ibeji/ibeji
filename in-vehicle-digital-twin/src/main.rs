// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::RwLock;
use proto::digitaltwin::digital_twin_server::DigitalTwinServer;
use proto::provider::provider_server::ProviderServer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

mod digitaltwin_impl;
mod provider_impl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = "[::1]:50010".parse()?;
    let provider_impl = provider_impl::ProviderImpl::default();
    let digitaltwin_impl =
        digitaltwin_impl::DigitalTwinImpl { entity_map: Arc::new(RwLock::new(HashMap::new())) };
    let server_future = Server::builder()
        .add_service(ProviderServer::new(provider_impl))
        .add_service(DigitalTwinServer::new(digitaltwin_impl))
        .serve(addr);

    server_future.await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
