// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::{sdv_v1 as sdv};
use log::{info, warn};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::{
    PublishRequest, PublishResponse, RespondRequest, RespondResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct ConsumerImpl {}

#[tonic::async_trait]
impl DigitalTwinConsumer for ConsumerImpl {
    /// Publish implementation.
    ///
    /// # Arguments
    /// * `request` - Publish request.
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let PublishRequest { entity_id, value } = request.into_inner();
        let value_json: serde_json::Value = serde_json::from_str(&value)
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        let massage_airbags_json =
            value_json.get(sdv::airbag_seat_massager::massage_airbags::NAME).unwrap();

        let massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE =
            serde_json::from_value(massage_airbags_json.clone()).unwrap();

        info!("Received a publish for entity id {entity_id} with the value: {massage_airbags:?}");

        let response = PublishResponse {};

        Ok(Response::new(response))
    }

    /// Respond implementation.
    ///
    /// # Arguments
    /// * `request` - Respond request.
    async fn respond(
        &self,
        request: Request<RespondRequest>,
    ) -> Result<Response<RespondResponse>, Status> {
        warn!("Got a respond request: {request:?}");

        Err(Status::unimplemented("respond has not been implemented"))
    }
}

#[cfg(test)]
mod consumer_impl_tests {
    use super::*;
    use digital_twin_model::Metadata;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Property {
        #[serde(rename = "MassageAirbags")]
        massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE,
        #[serde(rename = "$metadata")]
        metadata: Metadata,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponsePayload {}

    #[tokio::test]
    async fn publish_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-entity-id");

        let property: Property = Property {
            massage_airbags: Vec::new(),
            metadata: Metadata {
                model: sdv::airbag_seat_massager::massage_airbags::ID.to_string(),
            },
        };
        let property_json = serde_json::to_string(&property).unwrap();

        let request = tonic::Request::new(PublishRequest { entity_id, value: property_json });
        let result = consumer_impl.publish(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respond_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-entity-id");
        let response_id = String::from("some-response-id");

        let response_payload: ResponsePayload = ResponsePayload {};
        let response_payload_json = serde_json::to_string(&response_payload).unwrap();

        let request = tonic::Request::new(RespondRequest {
            entity_id,
            response_id,
            payload: response_payload_json,
        });
        let result = consumer_impl.respond(request).await;
        assert!(result.is_err());
    }
}
