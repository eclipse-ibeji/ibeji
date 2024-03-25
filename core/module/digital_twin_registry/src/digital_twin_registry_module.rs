// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use common::grpc_module::GrpcModule;
use core_protobuf_data_access::module::digital_twin_registry::v1::digital_twin_registry_server::DigitalTwinRegistryServer;

// use log::{debug, error, info};
use tonic::transport::server::RoutesBuilder;
// use tonic::{Request, Response, Status};

use crate::digital_twin_registry_impl::DigitalTwinRegistryImpl;

/// Digital Twin Registry Module.
#[derive(Clone, Debug)]
pub struct DigitalTwinRegistryModule {}

impl DigitalTwinRegistryModule {
    /// Creates a new instance of the DigitalTwinRegistryModule.
    pub async fn new() -> Result<Self, tonic::Status> {
        Ok(Self {})
    }
}

impl GrpcModule for DigitalTwinRegistryModule {
    /// Adds the gRPC services for this module to the server builder.
    ///
    /// # Arguments
    /// * `builder` - A tonic::RoutesBuilder that contains the grpc services to build.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder) {
        // Create the gRPC services.
        let digital_twin_registry_service =
            DigitalTwinRegistryServer::new(DigitalTwinRegistryImpl::default());

        builder.add_service(digital_twin_registry_service);
    }
}
