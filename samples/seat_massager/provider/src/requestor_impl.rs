// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use log::{debug, info, warn};
use parking_lot::{Mutex};
use samples_protobuf_data_access::async_rpc::v1::common::Status;
use samples_protobuf_data_access::async_rpc::v1::requestor::{
    requestor_server::Requestor, AskRequest, AskResponse, NotifyRequest,
    NotifyResponse,
};
use samples_protobuf_data_access::async_rpc::v1::responder::{
    responder_client::ResponderClient, AnswerRequest,
};
// use samples_protobuf_data_access::async_rpc::v1::requestor::requestor_client::RequestorClient;
// use samples_protobuf_data_access::async_rpc::v1::requestor::{AskRequest, AskResponse};
// use serde_derive::{Deserialize, Serialize};
// use std::pin::Pin;
use std::sync::Arc;
// use std::vec::Vec;
// use tokio_stream::Stream;
use tonic;

#[derive(Debug, Default)]
pub struct RequestorState {

}

#[derive(Debug, Default)]
pub struct RequestorImpl {
    pub state: Arc<Mutex<RequestorState>>,
}

impl RequestorImpl {
 
}

#[tonic::async_trait]
impl Requestor for RequestorImpl {

    /// Ask implementation.
    ///
    /// # Arguments
    /// * `request` - Ask request.
    async fn ask(
        &self,
        request: tonic::Request<AskRequest>,
    ) -> Result<tonic::Response<AskResponse>, tonic::Status> {
        let request_inner = request.into_inner();
        let responder_uri: String = request_inner.responder_uri.clone();
        let request_id: String = request_inner.request_id.clone();
        let payload: String = request_inner.payload.clone();

        info!("Received an ask request");

        info!("responder_uri: {responder_uri}");
        info!("request_id: {request_id}");
        info!("payload: {payload}");

        // let payload_json: serde_json::Value = serde_json::from_str(payload).unwrap();

        tokio::spawn(async move {
            // if entity_id == sdv::airbag_seat_massager::massage_airbags::ID {
                let client_result = ResponderClient::connect(responder_uri).await;
                if let Err(error_message) = client_result {
                    warn!("Unable to connect due to {error_message}");
                    return;
                }
                let mut client = client_result.unwrap();

                let response_payload: sdv::airbag_seat_massager::perform_step::response::TYPE = sdv::airbag_seat_massager::perform_step::response::TYPE {
                    status: sdv::airbag_seat_massager::status::TYPE {
                        code: 200,
                        message: "Ok".to_string(),
                    },
                };

                let response_payload_json: String = serde_json::to_string_pretty(&response_payload).unwrap();

                let answer_request = tonic::Request::new(AnswerRequest {
                    request_id,
                    payload: response_payload_json,
                });
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
            status: Some(Status{ code: 200, message: "Ok".to_string() })
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
