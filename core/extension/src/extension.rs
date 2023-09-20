// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{convert::Infallible, future::Future, net::SocketAddr};

use tonic::codegen::http::{request::Request, response::Response};
use tonic::{
    body::BoxBody,
    transport::{server::RoutesBuilder, Body, NamedService, Server},
};
use tower::{Service, ServiceBuilder};

// Extension references behind feature flags. Add any necessary extension references here.
// Start: Extension references.

// Add a new feature to all() so the use statement is active for the feature.
// ex. #[cfg(all(feature = "feature_1", feature = "feature_2"))]
#[cfg(all(feature = "managed_subscribe"))]
use common::{grpc_interceptor::GrpcInterceptorLayer, grpc_extension::GrpcExtension};

#[cfg(feature = "managed_subscribe")]
use crate::managed_subscribe::managed_subscribe_ext::ManagedSubscribeExt;

// End: Extension references.

/// Create and serve a tonic server with extensions enabled through the feature flags.
///
/// # Arguments
/// * `addr` - Socket address to host the services on.
/// * `base_service` - Core service that will be hosted.
///
/// # How to add an Extension service to this method:
/// 1. Add a block of code with the appropriate cfg feature flag.
/// 2. Create the `GrpcService` object within the block - if applicable.
/// 3. Create the `GrpocInterceptorLayer` object(s) within the block - if applicable.
/// 4. Call `.add_grpc_services()` on the `GrpcService` to add the gRPC server components to the
/// server builder.
/// 5. Call layer() for each `GrpcInterceptorLayer` object on the middleware chain and return that
/// from the block.
///
/// Note: It is expected that there is an extension service the implements `GrpcService`
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
    // Initialize services builder.
    let mut extensions_builder = RoutesBuilder::default();

    // Initialize middleware stack.
    let middleware = ServiceBuilder::new();

    // (1) Initialize the extension (interceptor and services) if the feature is enabled.
    #[cfg(feature = "managed_subscribe")]
    let middleware = {
        // (2) Initialize a new managed subscribe extension.
        let managed_subscribe_ext = ManagedSubscribeExt::new();

        // (3) Create interceptor layer to be added to the server.
        let managed_subscribe_layer =
            GrpcInterceptorLayer::new(Box::new(managed_subscribe_ext.create_interceptor()));

        // (4) Add extension services to routes builder.
        managed_subscribe_ext.add_grpc_services(&mut extensions_builder);

        // (5) Add layer(s) to middleware and return result.
        middleware.layer(managed_subscribe_layer)
    };

    // Construct the server.
    let mut builder = Server::builder()
        .layer(middleware.into_inner())
        .add_routes(extensions_builder.routes())
        .add_service(base_service);

    // Start the server.
    builder.serve(addr)
}
