// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::chariott::provider::v1::provider_service_server::ProviderServiceServer;
use core_protobuf_data_access::chariott::runtime::v1::chariott_service_client::ChariottServiceClient;
use core_protobuf_data_access::chariott::runtime::v1::{
    intent_registration, intent_service_registration, IntentRegistration,
    IntentServiceRegistration, RegisterRequest,
};
use core_protobuf_data_access::digital_twin::v1::digital_twin_server::DigitalTwinServer;
use env_logger::{Builder, Target};
use log::{debug, error, info, LevelFilter};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Status};
use url::Url;

mod digitaltwin_impl;
mod invehicle_digital_twin_config;
mod providerservice_impl;

pub const DIGITAL_TWIN_SERVICE_NAME: &str = "digital_twin";
pub const DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
pub const CHARIOTT_NAMESPACE_FOR_IBEJI: &str = "sdv.ibeji";

/// Register the digital twin service with Chariott.
///
/// # Arguments
/// * `chariott_url` - Chariott's URL.
/// * `invehicle_digital_twin_url` - In-vehicle Digital Twin Service's URL.
pub async fn register_digital_twin_service_with_chariott(
    chariott_url: &str,
    invehicle_digital_twin_url: &str,
) -> Result<(), Status> {
    let mut client = ChariottServiceClient::connect(chariott_url.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let service = Some(IntentServiceRegistration {
        name: DIGITAL_TWIN_SERVICE_NAME.to_string(),
        version: DIGITAL_TWIN_SERVICE_VERSION.to_string(),
        url: invehicle_digital_twin_url.to_string(),
        locality: intent_service_registration::ExecutionLocality::Local as i32,
    });

    let intents = vec![IntentRegistration {
        namespace: CHARIOTT_NAMESPACE_FOR_IBEJI.to_string(),
        intent: intent_registration::Intent::Discover as i32,
    }];

    let request = Request::new(RegisterRequest { service, intents });

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
    let chariott_url_option = settings.chariott_url;

    // Setup the HTTP server.
    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;
    let digitaltwin_impl = digitaltwin_impl::DigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    let invehicle_digital_twin_url = Url::parse(&invehicle_digital_twin_address)?;
    let providerservice_impl =
        providerservice_impl::ProviderServiceImpl::new(invehicle_digital_twin_url);
    let server_future = Server::builder()
        .add_service(DigitalTwinServer::new(digitaltwin_impl))
        .add_service(ProviderServiceServer::new(providerservice_impl))
        .serve(addr);
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");

    // Register the digital twin service with Chariott if Chariott's URL was provided in the config.
    // Note: We are not using Chariott's announce, and therefore the digital twin serice will be forcibly unregistered
    //       after 15 seconds unless the CHARIOTT_REGISTRY_TTL_SECS environment variable is set. Please make sure that
    //       it is set (and exported) in the shell running Chariott before Chariott has started.
    if chariott_url_option.is_some() {
        match register_digital_twin_service_with_chariott(
            &chariott_url_option.unwrap(),
            &invehicle_digital_twin_address,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(error) => {
                error!("Failed to register this service with Chariott: '{error}'");
                Err(error)?
            }
        };
        info!("This service is now registered with Chariott.");
    } else {
        info!("This service is not using Chariott.");
    }

    server_future.await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
