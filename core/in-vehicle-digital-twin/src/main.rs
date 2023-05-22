// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::digital_twin::v1::digital_twin_server::DigitalTwinServer;
use core_protobuf_data_access::chariott::provider::v1::provider_service_server::ProviderServiceServer;
use core_protobuf_data_access::chariott::runtime::v1::chariott_service_client::ChariottServiceClient;
use core_protobuf_data_access::chariott::runtime::v1::{intent_registration, IntentRegistration, intent_service_registration, IntentServiceRegistration, RegisterRequest};
use env_logger::{Builder, Target};
use log::{debug, info, LevelFilter};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Status};
use tonic::transport::Server;
use url::Url;

mod digitaltwin_impl;
mod providerservice_impl;

const IN_VEHICLE_DIGITAL_TWIN_AUTHORITY: &str = "0.0.0.0:5010";

pub async fn register_ibeji_services_with_chariott(digital_twin_url: &str) -> Result<(), Status> {

    let chariott_url = "http://0.0.0.0:4243";

    let mut client = ChariottServiceClient::connect(chariott_url.to_string()).await.map_err(|e|Status::internal(e.to_string()))?;

    let service = Some(IntentServiceRegistration {
        name: "digital-twin".to_string(),
        version: "1.0".to_string(),
        url: digital_twin_url.to_string(),
        locality: intent_service_registration::ExecutionLocality::Local as i32,
    });

    let intents = vec![
        IntentRegistration {
            namespace: "sdv.ibeji".to_string(),
            intent: intent_registration::Intent::Discover as i32,
        },
    ];

    let request = Request::new(RegisterRequest {
        service,
        intents,
    });

    let response = client.register(request).await;

    if !response.is_ok() {
        return Err(Status::internal("Chariott register request failed"));
    }

    info!("{:?}", response.unwrap().into_inner());

    return Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Setup the HTTP server.
    let addr: SocketAddr = IN_VEHICLE_DIGITAL_TWIN_AUTHORITY.parse()?;
    let digitaltwin_impl = digitaltwin_impl::DigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let in_vehicle_digital_twin_address = format!("http://{IN_VEHICLE_DIGITAL_TWIN_AUTHORITY}"); // Devskim: ignore DS137138    
    let in_vehicle_digital_twin_url = Url::parse(&in_vehicle_digital_twin_address)?;
    let providerservice_impl = providerservice_impl::ProviderServiceImpl::new(in_vehicle_digital_twin_url);
    let server_future =
        Server::builder()
            .add_service(DigitalTwinServer::new(digitaltwin_impl))
            .add_service(ProviderServiceServer::new(providerservice_impl))
            .serve(addr);
    info!("The HTTP server is listening on address '{in_vehicle_digital_twin_address}'");

    register_ibeji_services_with_chariott(&in_vehicle_digital_twin_address).await?;

    server_future.await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
