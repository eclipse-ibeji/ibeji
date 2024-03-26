// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use common::grpc_module::GrpcModule;
use core_protobuf_data_access::async_rpc::v1::respond::respond_server::RespondServer;
use core_protobuf_data_access::module::digital_twin_graph::v1::digital_twin_graph_server::DigitalTwinGraphServer;
// use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::broadcast;
use tonic::transport::server::RoutesBuilder;

use crate::digital_twin_graph_impl::DigitalTwinGraphImpl;
use crate::respond_impl::RespondImpl;

/// Digital Twin Graph Module.
#[derive(Clone, Debug)]
pub struct DigitalTwinGraphModule {}

impl DigitalTwinGraphModule {
    /// Creates a new instance of the DigitalTwinGraphModule.
    pub async fn new() -> Result<Self, tonic::Status> {
        Ok(Self {})
    }
}

impl GrpcModule for DigitalTwinGraphModule {
    /// Adds the gRPC services for this module to the server builder.
    ///
    /// # Arguments
    /// * `builder` - A tonic::RoutesBuilder that contains the grpc services to build.
    fn add_grpc_services(&self, builder: &mut RoutesBuilder) {
        // Note: The authority is hardcoded for now, but it should be configurable in the future.
        let invehicle_digital_twin_authority = "0.0.0.0:5010";
        let invehicle_digital_twin_uri = format!("http://{invehicle_digital_twin_authority}"); // Devskim: ignore DS137138
        let respond_authority = "0.0.0.0:5010";
        let respond_uri = format!("http://{respond_authority}"); // Devskim: ignore DS137138

        let (tx, _rx) = broadcast::channel(100);
        let tx = Arc::new(tx);

        // Setup the respond service.
        let respond_impl = RespondImpl::new(tx.clone());
        let respond_service = RespondServer::new(respond_impl);

        // Setup the digital twin graph service.
        let digital_twin_graph_impl =
            DigitalTwinGraphImpl::new(&invehicle_digital_twin_uri, &respond_uri, tx);
        let digital_twin_graph_service = DigitalTwinGraphServer::new(digital_twin_graph_impl);

        builder.add_service(digital_twin_graph_service);
        builder.add_service(respond_service);
    }
}
