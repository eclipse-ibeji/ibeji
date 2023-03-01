// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::info;
use proto::consumer::consumer_server::Consumer;
use proto::consumer::{PublishRequest, PublishResponse, RespondRequest, RespondResponse};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct ConsumerImpl {}

#[tonic::async_trait]
impl Consumer for ConsumerImpl {
    /// Publish implementation.
    ///
    /// # Arguments
    /// * `request` - Publish request.
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let PublishRequest { entity_id, value } = request.into_inner();

        info!("Received a publish for entity id {entity_id} with the value {value}");

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
        let RespondRequest { entity_id, response_id, payload } = request.into_inner();

        info!("Received a respond for entity id {entity_id} with the response id {response_id} and the payload '{payload}'");

        let response = RespondResponse {};

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod consumer_impl_tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn publish_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-id");
        let value = String::from("some-value");

        let request = tonic::Request::new(PublishRequest { entity_id, value });
        let result = consumer_impl.publish(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respond_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-id");
        let response_id = Uuid::new_v4().to_string();
        let payload = String::from("some-payload");

        let request = tonic::Request::new(RespondRequest { entity_id, response_id, payload });
        let result = consumer_impl.respond(request).await;
        assert!(result.is_ok());
    }
}
