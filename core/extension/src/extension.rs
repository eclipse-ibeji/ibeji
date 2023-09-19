// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{convert::Infallible, net::SocketAddr, future::Future};

use common::grpc_interceptor::GrpcInterceptorLayer;
use tonic::{transport::{Server, NamedService, Body, server::RoutesBuilder}, body::BoxBody};
use tonic::codegen::http::{request::Request, response::Response};
use tower::{ServiceBuilder, Service};

// Extension references behind feature flags. Add any necessary extension references here.
// Start: Extension references.

#[cfg(feature = "managed_subscribe")]
use crate::managed_subscribe::managed_subscribe_ext::ManagedSubscribeExt;

// End: Extension references.

/// Trait that must be implemented for an extension to add a grpc service to the hosted server.
pub trait GrpcExtensionService {
    /// Function to add necessary extension services to the routers builder.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder);
}

/// Create and serve a tonic server with extensions enabled through the feature flags.
///
/// # Arguments
/// * `addr` - Socket address to host the services on.
/// * `base_service` - Core service that will be hosted.
///
/// # How to add an Extension service to this method:
/// 1. Add a mutable option for the extension `GrpcInterceptorLayer` - if applicable.
/// 2. Add a block of code with the appropriate cfg feature flag. Create the `GrpcExtensionService`
/// object and `GrpcInterceptorLayer` object within the block. Call `.add_grpc_services()` to add
/// the gRPC server components to the server builder.
/// 3. Add an `.option_layer()` to the middleware `ServiceBuilder` with the option created in (1).
///
/// Note: It is expected that there is an extension service the implements `GrpcExtensionService`
/// (if there is a gRPC server component) and that a feature flag created for the extension.
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
    let mut extensions_builder = RoutesBuilder::default();

    // (1) Add option for an interceptor layer. This will default to none if the feature for the
    // extension is not enabled.
    let mut managed_subscribe_layer: Option<GrpcInterceptorLayer> = None;

    // (2) Initialize the extension (interceptor and services) if the feature is enabled.
    #[cfg(feature = "managed_subscribe")]
    {
        // Initialize a new managed subscribe extension.
        let managed_subscribe_ext = ManagedSubscribeExt::new();

        // Create interceptor layer to be added to the server.
        managed_subscribe_layer = Some(GrpcInterceptorLayer::new(Box::new(managed_subscribe_ext.create_interceptor())));

        // Add extension services to routes builder.
        managed_subscribe_ext.add_grpc_services(&mut extensions_builder);
    }

    // (3) Build the middleware for the server.
    let middleware = ServiceBuilder::new()
        .option_layer(managed_subscribe_layer)
        .into_inner(); // Unwraps the ServiceBuilder for less decoration.

    // Construct the server.
    let mut builder = Server::builder()
        .layer(middleware)
        .add_routes(extensions_builder.routes())
        .add_service(base_service);

    // Start the server.
    builder.serve(addr)
}
