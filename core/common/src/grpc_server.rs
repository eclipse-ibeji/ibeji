// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

use tonic::transport::server::Router;
use tonic::transport::{server::RoutesBuilder, Server};
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;

use crate::grpc_module::GrpcModule;

/// Grpc Server struct that builds multiple services and layers.
pub struct GrpcServer<L> {
    address: SocketAddr,
    pub modules: RoutesBuilder,
    pub middleware: ServiceBuilder<L>,
}

impl GrpcServer<Identity> {
    /// Creates a new GrpcServer
    ///
    /// # Arguments
    /// * `address` - The address the server will be hosted on.
    pub fn new(address: SocketAddr) -> Self {
        GrpcServer { address, modules: RoutesBuilder::default(), middleware: ServiceBuilder::new() }
    }
}

impl<L> GrpcServer<L> {
    /// Adds a module (collection of grpc services and interceptors) for the server to host.
    /// Returns a newly decorated GrpcServer with the added module.
    ///
    /// # Arguments
    /// * `middleware` - The middleware from the current server + any interceptors for the module
    ///                  added with `.layer()`.
    /// * `module` - Boxed GrpcModule to be added to the server.
    pub fn add_module<S>(
        &mut self,
        middleware: ServiceBuilder<S>,
        module: Box<dyn GrpcModule>,
    ) -> GrpcServer<S> {
        module.add_grpc_services(&mut self.modules);

        GrpcServer { address: self.address, modules: self.modules.clone(), middleware }
    }

    /// Constructs the added modules and layers into a server to host.
    pub fn construct_server(&self) -> Router<Stack<L, Identity>>
    where
        L: Clone,
    {
        // Construct the server.
        Server::builder()
            .layer(self.middleware.clone().into_inner())
            .add_routes(self.modules.clone().routes())
    }
}
