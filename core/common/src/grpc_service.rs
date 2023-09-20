// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use tonic::transport::server::RoutesBuilder;

/// Trait that must be implemented for a service to add a grpc service to the hosted server.
pub trait GrpcService {
    /// Function to add necessary services to the server builder.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder);
}
