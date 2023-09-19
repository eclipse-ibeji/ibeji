// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{convert::Infallible, net::SocketAddr, future::Future};

use common::grpc_interceptor::GrpcInterceptorLayer;
use tonic::{transport::{Server, NamedService, Body, server::Router}, body::BoxBody};
use tonic::codegen::http::{request::Request, response::Response};
use tower::{ServiceBuilder, Service};

// Dependencies used by certain extensions. Add an extension by adding an entry to all();
// ex. #[cfg(all(feature = "extension_1", feature = "extension_2"))]
#[cfg(all(feature = "managed_subscribe"))]
use parking_lot::RwLock;
#[cfg(all(feature = "managed_subscribe"))]
use std::sync::Arc;

// Extensions behind feature flags.
#[cfg(feature = "managed_subscribe")]
use crate::managed_subscribe::{self, managed_subscribe_interceptor::ManagedSubscribeInterceptor};

#[allow(dead_code)]
type NoneType = String;

#[cfg(not(feature = "managed_subscribe"))]
type ManagedSubscribeExt = NoneType;

#[cfg(feature = "managed_subscribe")]
type ManagedSubscribeExt = managed_subscribe::managed_subscribe_ext::ManagedSubscribeExt;

/// Trait that must be implemented for an extension to add a grpc service to the hosted server.
pub trait GrpcExtensionService: Send {
    // fn get_services(&self) -> Routes;
    // Function to add necessary extension services to the server builder.
    fn add_services<L>(&self, builder: Router<L>) -> Router<L>;
}

/// Trait that extends the tonic::Router struct to add an optional GrpcExtensionService.
pub trait GrpcServiceBuilderExtension<L> {
    fn add_optional_extension_service<S>(self, service: Option<S>) -> Router<L> where S: GrpcExtensionService;
}

/// Implementation that adds an optional GrpcExtensionService to a tonic server.
impl <L> GrpcServiceBuilderExtension<L> for Router<L> {
    fn add_optional_extension_service<S>(self, service: Option<S>) -> Router<L>
    where S: GrpcExtensionService
    {
        let mut builder = self;

        if let Some(svc) = service {
            builder = svc.add_services(builder);
        }

        builder
    }
}

/// Create and serve a tonic server with extensions enabled through the feature flags.
///
/// # Arguments
/// * `addr` - Socket address to host the services on.
/// * `base_service` - Core service that will be hosted.
#[allow(unused_assignments, unused_mut)] // Necessary when no interceptors are built.
pub fn serve_with_extensions<S>(
    addr: SocketAddr,
    base_service: S,
) -> impl Future<Output = Result<(), tonic::transport::Error>>
where
    S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    // Add option for an interceptor layer. This will default to none if the feature for the extension is not enabled.
    let mut managed_subscribe_layer: Option<GrpcInterceptorLayer> = None;

    // Add option for an extension. This will default to none if the feature for the extension is not enabled.
    let mut managed_subscribe_ext: Option<ManagedSubscribeExt> = None;

    // Initialize the interceptor if the feature is enabled.
    #[cfg(feature = "managed_subscribe")]
    {
        let store = managed_subscribe::managed_subscribe_store::SubscriptionStore::new();
        let store_handle = Arc::new(RwLock::new(store));

        managed_subscribe_layer = Some(GrpcInterceptorLayer::new(Box::new(ManagedSubscribeInterceptor::new(store_handle.clone()))));
        
        managed_subscribe_ext = Some(ManagedSubscribeExt::new(store_handle));
    }

    // build the middleware for the server.
    let middleware = ServiceBuilder::new()
        .option_layer(managed_subscribe_layer);

    // Construct the server.
    let mut builder = Server::builder()
        .layer(middleware)
        .add_service(base_service)
        .add_optional_extension_service(managed_subscribe_ext);

    // Start the server.
    builder.serve(addr)
}
