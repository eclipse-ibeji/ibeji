// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// use digital_twin_model::{sdv_v1 as sdv};
use log::{debug, info};
use samples_protobuf_data_access::async_rpc::v1::common::Status;
use samples_protobuf_data_access::async_rpc::v1::respond::respond_server::Respond;
use samples_protobuf_data_access::async_rpc::v1::respond::{AnswerRequest, AnswerResponse};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct RespondImpl {
    pub tx: mpsc::Sender<AnswerRequest>,
}

impl RespondImpl {
    pub fn new(tx: mpsc::Sender<AnswerRequest>) -> RespondImpl {
        RespondImpl { tx }
    }
}

#[tonic::async_trait]
impl Respond for RespondImpl {
    /// Answer implementation.
    ///
    /// # Arguments
    /// * `request` - Respond request.
    async fn answer(
        &self,
        request: tonic::Request<AnswerRequest>,
    ) -> Result<tonic::Response<AnswerResponse>, tonic::Status> {
        info!("Received an answer request");

        if let Err(err_msg) = self.tx.send(request.into_inner()).await {
            return Err(tonic::Status::internal(format!(
                "Failed to send the answer request due to {err_msg}"
            )));
        }

        debug!("Completed the answer request.");

        Ok(tonic::Response::new(AnswerResponse {
            status: Some(Status { code: 200, message: "Ok".to_string() }),
        }))
    }
}
