// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::chariott::service_discovery::core::v1::service_registry_client::ServiceRegistryClient;
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    RegisterRequest, ServiceMetadata,
};
// use core_protobuf_data_access::invehicle_digital_twin::v1;
use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
use core::task::Poll;
use env_logger::{Builder, Target};
use log::{debug, error, info, LevelFilter};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::Context;
use tonic::transport::Server;
use tonic::{Request, Status};
use tower::{Layer, Service, ServiceBuilder};

mod invehicle_digital_twin_config;
mod invehicle_digital_twin_impl;

const INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE: &str = "sdv.ibeji";
const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "invehicle_digital_twin";
const INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND: &str = "grpc+proto";
const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE: &str = "https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/digital_twin/v1/digital_twin.proto";

#[derive(Clone)]
pub struct MyLayer {
}

impl MyLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for MyLayer {
    type Service = MyService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyService {
            service
        }
    }
}

#[derive(Clone)]
pub struct MyService<S> {
    service: S,
}

impl<S> MyService<S>
{
/*
    /// Returns the underlying `Layer` implementation.
    fn into_inner(self) -> S {
        info!("into_inner called");
        self.service
    }
*/

    fn displayable_type_of<T>(_: &T) -> String {
        format!("{}", std::any::type_name::<T>())
    } 
}



// https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
// https://stackoverflow.com/questions/76625360/how-can-i-get-grpc-status-code-in-rust-tonic-middleware?rq=2
// https://github.com/hyperium/tonic/discussions/815



/*
impl<S, Request> Service<Request> for MyService<S>
where
    S: Service<Request>,
    Request: Debug,
*/
impl<S, Body> Service<http::request::Request<Body>> for MyService<S>
where
    S: Service<http::request::Request<Body>>,
    // Body: Debug
    Body: http_body::Body + std::marker::Send + std::marker::Sync + 'static
    // Note: Body has type tonic::transport::Body.
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        info!("poll_ready");
        self.service.poll_ready(cx)
    }

    // fn call(&mut self, request: Request) -> Self::Future {
    fn call(&mut self, request: http::request::Request<Body>) -> Self::Future {
        info!("request = {:?}", request.method());
        info!("request type = {}", Self::displayable_type_of(&request));
        info!("method = {}", request.method());
        info!("uri = {}", request.uri());
        // let boxed_body = request.body().boxed();
        self.service.call(request)
    }   
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The In-Vehicle Digital Twin Service has started.");

    // Load the config.
    let settings = invehicle_digital_twin_config::load_settings();
    let invehicle_digital_twin_authority = settings.invehicle_digital_twin_authority;
    let chariott_uri_option = settings.chariott_uri;

/*
    let my_layer = MyLayer{};

    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .layer(my_layer)
        .into_inner();

    // Setup the HTTP server.
    let addr: SocketAddr = invehicle_digital_twin_authority.parse()?;
    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };
    let invehicle_digital_twin_address = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
    let server_future = Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr);
    info!("The HTTP server is listening on address '{invehicle_digital_twin_address}'");
*/

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

    let layer = ServiceBuilder::new()
        .layer(MyLayer::new());

    Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr)
        .await?;

    debug!("The Digital Twin Service has completed.");

    Ok(())
}
