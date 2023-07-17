// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::{sdv_v1 as sdv};
use log::{info, warn};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumer;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::{
    PublishRequest, PublishResponse, RespondRequest, RespondResponse,
};
use serde_json;
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
        let j: serde_json::Value = serde_json::from_str(&value).unwrap();
        let j_prop = j.get(sdv::airbag_seat_massager::massage_airbags::NAME).unwrap();

        let massage_airbags: sdv::airbag_seat_massager::massage_airbags::TYPE = serde_json::from_value(j_prop.clone()).unwrap();

        println!("{:?}", massage_airbags);

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

    #[tokio::test]
    async fn publish_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-entity-id");
        let value = String::from("some-value");

        let request = tonic::Request::new(PublishRequest { entity_id, value });
        let result = consumer_impl.publish(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn respond_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-entity-id");
        let response_id = String::from("some-response-id");
        let payload = String::from("some-payload");        

        let request = tonic::Request::new(RespondRequest { entity_id, response_id, payload });
        let result = consumer_impl.respond(request).await;
        assert!(result.is_err());
    }    
}
