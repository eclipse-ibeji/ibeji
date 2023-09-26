// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use tonic::transport::server::RoutesBuilder;

/// Trait that must be implemented for a module to add one or more grpc services to the hosted
/// server. A GrpcModule may also implement one or more GrpcInterceptor objects and share state.
pub trait GrpcModule {
    /// Function to add necessary services to the server builder.
    ///
    /// # Arguments
    /// * `builder` - A tonic::RoutesBuilder that contains the grpc services to build.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder);
}
