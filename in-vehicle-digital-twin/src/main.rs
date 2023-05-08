// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::RwLock;
use proto::digitaltwin::digital_twin_server::DigitalTwinServer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

mod digitaltwin_impl;

const IN_VEHICLE_DIGITAL_TWIN_ADDR: &str = "[::1]:50010";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = IN_VEHICLE_DIGITAL_TWIN_ADDR.parse()?;
    let digitaltwin_impl = digitaltwin_impl::DigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let server_future =
        Server::builder().add_service(DigitalTwinServer::new(digitaltwin_impl)).serve(addr);
    info!("The HTTP server is listening on address '{IN_VEHICLE_DIGITAL_TWIN_ADDR}'");

    server_future.await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
