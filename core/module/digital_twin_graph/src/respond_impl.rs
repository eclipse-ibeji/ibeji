// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::async_rpc::v1::respond::respond_server::Respond;
use core_protobuf_data_access::async_rpc::v1::respond::{AnswerRequest, AnswerResponse};
use log::debug;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct RespondImpl {
    pub tx: Arc<broadcast::Sender<AnswerRequest>>,
}

impl RespondImpl {
    /// Create a new instance of a RespondImpl.
    ///
    /// # Arguments
    /// * `tx` - The sender for the asynchronous channel for AnswerRequests.
    pub fn new(tx: Arc<broadcast::Sender<AnswerRequest>>) -> RespondImpl {
        RespondImpl { tx }
    }
}

#[tonic::async_trait]
impl Respond for RespondImpl {
    /// Answer implementation.
    ///
    /// # Arguments
    /// * `request` - The answer's request.
    async fn answer(
        &self,
        request: tonic::Request<AnswerRequest>,
    ) -> Result<tonic::Response<AnswerResponse>, tonic::Status> {
        debug!("Received an answer request");

        let tx = Arc::clone(&self.tx);

        // Send the request to the channel.
        if let Err(err_msg) = tx.send(request.into_inner()) {
            return Err(tonic::Status::internal(format!(
                "Failed to send the answer request due to: {err_msg}"
            )));
        }

        debug!("Completed the answer request.");

        Ok(tonic::Response::new(AnswerResponse {}))
    }
}
