// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Module references behind feature flags. Add any necessary module references here.
// Start: Module references.

#[cfg(feature = "managed_subscribe")]
use managed_subscribe::managed_subscribe_module::ManagedSubscribeModule;

// End: Module references.

#[allow(unused_imports)]
use common::grpc_interceptor::GrpcInterceptorLayer;

use common::grpc_server::GrpcServer;
use core_protobuf_data_access::chariott::service_discovery::core::v1::service_registry_client::ServiceRegistryClient;
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    RegisterRequest, ServiceMetadata,
};
use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
use env_logger::{Builder, Target};
use futures::Future;
use log::{debug, error, info, LevelFilter};
use parking_lot::RwLock;
use std::boxed::Box;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::{Body, NamedService};
use tonic::{Request, Status};
use tower::layer::util::Identity;
use tower::Service;

mod invehicle_digital_twin_config;
mod invehicle_digital_twin_impl;

const INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE: &str = "sdv.ibeji";
const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "invehicle_digital_twin";
const INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND: &str = "grpc+proto";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE: &str = "https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/digital_twin/v1/digital_twin.proto";

/// Register the invehicle digital twin service with Chariott.
///
/// # Arguments
/// * `chariott_uri` - Chariott's URI.
/// * `invehicle_digital_twin_uri` - In-vehicle Digital Twin Service's URI.
async fn register_invehicle_digital_twin_service_with_chariott(
    chariott_uri: &str,
    invehicle_digital_twin_uri: &str,
) -> Result<(), Status> {
    let mut client = ServiceRegistryClient::connect(chariott_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let service = Some(ServiceMetadata {
        namespace: INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE.to_string(),
        name: INVEHICLE_DIGITAL_TWIN_SERVICE_NAME.to_string(),
        version: INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION.to_string(),
        uri: invehicle_digital_twin_uri.to_string(),
        communication_kind: INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND.to_string(),
        communication_reference: INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE.to_string(),
    });

    let request = Request::new(RegisterRequest { service });

    client
        .register(request)
        .await
        .map_err(|_| Status::internal("Chariott register request failed"))?;

    Ok(())
}

/// Builds the enabled modules for the grpc server and starts the server.
///
/// # Arguments
/// * `addr` - The address the server will be hosted on.
/// * `base_service` - The core service that will be hosted.
///
/// # How to add a Module to this method:
/// 1. Add a block of code with the appropriate cfg feature flag.
/// 2. Create the `GrpcModule` object within the block - if applicable.
/// 3. Create the `GrpcInterceptorLayer` object(s) within the block - if applicable.
/// 4. Add the grpc interceptors to the middleware stack with `.layer()`.
/// 5. Call and return from the block `.add_module()` on the server with the updated middleware and
/// module.
#[allow(unused_assignments, unused_mut)] // Necessary when no extra modules are built.
fn build_server_and_serve<S>(
    addr: SocketAddr,
    base_service: S,
) -> impl Future<Output = Result<(), tonic::transport::Error>>
where
    S: Service<http::Request<Body>, Response = http::Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    let mut server: GrpcServer<Identity> = GrpcServer::new(addr);

    #[cfg(feature = "managed_subscribe")]
    // (1) Adds the Managed Subscribe module to the service.
    let server = {
        // (2) Initialize the Managed Subscribe module, which implements GrpcModule.
        let managed_subscribe_module = ManagedSubscribeModule::new();

        // (3) Create interceptor layer to be added to the server.
        let managed_subscribe_layer =
            GrpcInterceptorLayer::new(Box::new(managed_subscribe_module.create_interceptor()));

        // (4) Add the interceptor(s) to the middleware stack.
        let current_middleware = server.middleware.clone();
        let new_middleware = current_middleware.layer(managed_subscribe_layer);

        // (5) Add the module with the updated middleware stack to the server.
        server.add_module(new_middleware, Box::new(managed_subscribe_module))
    };

    // Construct the server.
    let builder = server.construct_server().add_service(base_service);

    // Start the server.
    builder.serve(addr)
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

    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;

    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");

    // Register the invehicle digital twin service with Chariott if Chariott's URI was provided in the config.
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

    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };

    let base_service = InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl);

    // Build and start the grpc server.
    build_server_and_serve(addr, base_service).await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
