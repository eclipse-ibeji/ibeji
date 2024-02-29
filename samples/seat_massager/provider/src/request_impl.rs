// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v2 as sdv;
use log::{debug, info, warn};
use parking_lot::Mutex;
use samples_protobuf_data_access::async_rpc::v1::common::Status;
use samples_protobuf_data_access::async_rpc::v1::request::{
    request_server::Request, AskRequest, AskResponse, NotifyRequest, NotifyResponse,
};
use samples_protobuf_data_access::async_rpc::v1::respond::{
    respond_client::RespondClient, AnswerRequest,
};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct RequestState {}

#[derive(Debug, Default)]
pub struct RequestImpl {
    pub state: Arc<Mutex<RequestState>>,
}

impl RequestImpl {}

#[tonic::async_trait]
impl Request for RequestImpl {
    /// Ask implementation.
    ///
    /// # Arguments
    /// * `request` - Ask request.
    async fn ask(
        &self,
        request: tonic::Request<AskRequest>,
    ) -> Result<tonic::Response<AskResponse>, tonic::Status> {
        let request_inner = request.into_inner();
        let respond_uri: String = request_inner.respond_uri.clone();
        let request_id: String = request_inner.request_id.clone();
        let payload: String = request_inner.payload.clone();

        info!("Received an ask request");

        info!("respond_uri: {respond_uri}");
        info!("request_id: {request_id}");
        info!("payload: {payload}");

        // let payload_json: serde_json::Value = serde_json::from_str(payload).unwrap();

        tokio::spawn(async move {
            // if entity_id == sdv::airbag_seat_massager::massage_airbags::ID {
            let client_result = RespondClient::connect(respond_uri).await;
            if let Err(error_message) = client_result {
                warn!("Unable to connect due to {error_message}");
                return;
            }
            let mut client = client_result.unwrap();

            let response_payload: sdv::airbag_seat_massager::perform_step::response::TYPE =
                sdv::airbag_seat_massager::perform_step::response::TYPE {
                    status: sdv::airbag_seat_massager::status::TYPE {
                        code: 200,
                        message: "Ok".to_string(),
                    },
                    ..Default::default()
                };

            let response_payload_json: String =
                serde_json::to_string_pretty(&response_payload).unwrap();

            let answer_request =
                tonic::Request::new(AnswerRequest { request_id, payload: response_payload_json });
            let response = client.answer(answer_request).await;
            if let Err(status) = response {
                warn!("Answer failed: {status:?}");
            }
            // } else {
            //    warn!("The entity id {entity_id} is not recognized.");
            // }
        });

        debug!("Completed the ask request.");

        Ok(tonic::Response::new(AskResponse {
            status: Some(Status { code: 200, message: "Ok".to_string() }),
        }))
    }

    /// Notify implementation.
    ///
    /// # Arguments
    /// * `request` - Notify request.
    async fn notify(
        &self,
        request: tonic::Request<NotifyRequest>,
    ) -> Result<tonic::Response<NotifyResponse>, tonic::Status> {
        warn!("Got a notify request: {request:?}");

        Err(tonic::Status::unimplemented("notify has not been implemented"))
    }
}
