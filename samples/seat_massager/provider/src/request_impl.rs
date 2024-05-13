// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use samples_protobuf_data_access::async_rpc::v1::request::{
    request_server::Request, AskRequest, AskResponse, NotifyRequest, NotifyResponse,
};
use samples_protobuf_data_access::async_rpc::v1::respond::{
    respond_client::RespondClient, AnswerRequest,
};
use seat_massager_common::{status, TargetedPayload};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct RequestState {}

#[derive(Debug, Default)]
pub struct RequestImpl {
    pub state: Arc<Mutex<RequestState>>,
}

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
        let ask_id: String = request_inner.ask_id.clone();
        let payload: String = request_inner.payload.clone();

        info!("Received an ask request");
        info!("  respond_uri: {respond_uri}");
        info!("  ask_id: {ask_id}");

        // Deserialize the targeted payload.
        let targeted_payload_json: TargetedPayload = serde_json::from_str(&payload).unwrap();

        info!("  instance_id: {}", targeted_payload_json.instance_id);
        info!("  member_path: {}", targeted_payload_json.member_path);
        info!("  operation: {}", targeted_payload_json.operation);
        info!("  payload: {}", targeted_payload_json.payload);

        // Deserialize the request payload.
        let request_payload_json: serde_json::Value =
            serde_json::from_str(&targeted_payload_json.payload)
                .map_err(|error| tonic::Status::invalid_argument(error.to_string()))?;

        // Extract the type_id from the request payload.
        let type_id_json: serde_json::Value = request_payload_json.get("@type").unwrap().clone();
        let type_id: String = serde_json::from_value(type_id_json.clone()).unwrap();

        // Check to make sure that the type_id is for a perform_request request.
        if type_id != sdv::airbag_seat_massager::perform_step::request::ID {
            return Err(tonic::Status::invalid_argument(format!("Unexpected type_id '{type_id}'")));
        }

        // Asynchronously perform the step.
        tokio::spawn(async move {
            let client_result = RespondClient::connect(respond_uri).await;
            if let Err(error_message) = client_result {
                error!("Unable to connect due to {error_message}");
                return;
            }
            let mut client = client_result.unwrap();

            // Extract the request from the request payload.
            let perform_step_request_opt: Option<
                sdv::airbag_seat_massager::perform_step::request::PAYLOAD_TYPE,
            > = serde_json::from_value(request_payload_json)
                .expect("Failed to deserialize the request.");
            if perform_step_request_opt.is_none() {
                error!("Failed to deserialize the request.");
                return;
            }
            let perform_step_request = perform_step_request_opt.unwrap();

            info!("Performing the step: {:?}", perform_step_request.step);

            // Prepare the perform_step response payload.
            let response_payload: sdv::airbag_seat_massager::perform_step::response::PAYLOAD_TYPE =
                sdv::airbag_seat_massager::perform_step::response::PAYLOAD_TYPE {
                    status: sdv::airbag_seat_massager::status::SCHEMA_TYPE {
                        code: status::ok::CODE,
                        message: status::ok::MESSAGE.to_string(),
                    },
                    ..Default::default()
                };

            // Serilaize the response payload.
            let response_payload_json: String =
                serde_json::to_string_pretty(&response_payload).unwrap();

            let answer_request =
                tonic::Request::new(AnswerRequest { ask_id, payload: response_payload_json });

            // Send the answer.
            let response = client.answer(answer_request).await;
            if let Err(status) = response {
                error!("Answer failed: {status:?}");
            }
        });

        debug!("Completed the ask request.");

        Ok(tonic::Response::new(AskResponse {}))
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
