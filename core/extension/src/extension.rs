// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{convert::Infallible, net::SocketAddr, future::Future};

use common::grpc_interceptor::GrpcInterceptorLayer;
use tonic::{transport::{Server, NamedService, Body, server::Router}, body::BoxBody};
use tonic::codegen::http::{request::Request, response::Response};
use tower::{ServiceBuilder, Service};

// Extensions behind feature flags.
#[cfg(feature = "managed_subscribe")]
use crate::managed_subscribe::{ managed_subscribe_ext::ManagedSubscribeExt, managed_subscribe_interceptor::ManagedSubscribeInterceptor};

/// Trait that must be implemented for an extension to add a grpc service to the hosted server.
pub trait ExtensionService {
    /// Function to add necessary extension services to the server builder.
    fn add_services<L>(&self, builder: Router<L>) -> Router<L> where L: Clone;
}

/// Trait for a tonic Router to add all the grpc extension services at once.
trait ExtensionServer<L> {
    fn add_extension_services(self) -> Router<L> where L: Clone;
}

impl <L> ExtensionServer<L> for Router<L> {
    /// Helper function to add extension services to main hosted service.
    #[allow(unused_mut)] // Necessary when no extensions are built.
    fn add_extension_services(self) -> Router<L> where L: Clone {
        let mut builder = self;

        #[cfg(feature = "managed_subscribe")]
        {
            builder = ManagedSubscribeExt::new().add_services(builder);
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
    // Add option for an interceptor layer. This will default to none if the feature for the extensions is not enabled.
    let mut managed_subscribe_layer: Option<GrpcInterceptorLayer> = None;

    // Initialize the interceptor if the feature is enabled.
    #[cfg(feature = "managed_subscribe")]
    {
        managed_subscribe_layer = Some(GrpcInterceptorLayer::new(ManagedSubscribeInterceptor::sample_grpc_interceptor_factory));
    }

    // build the middleware for the server.
    let middleware = ServiceBuilder::new()
        .option_layer(managed_subscribe_layer);

    // Construct the server.
    Server::builder()
        .layer(middleware)
        .add_service(base_service)
        .add_extension_services()
        .serve(addr)
}
