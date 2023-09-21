// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use tonic::transport::server::RoutesBuilder;

/// Trait that must be implemented for an extension to add a grpc service to the hosted server.
/// Note: This trait may be renamed in the future.
pub trait GrpcExtension {
    /// Function to add necessary services to the server builder.
    ///
    /// # Arguments
    /// * `builder` - A tonic::RoutesBuilder that contains the grpc services to build.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder);
}
