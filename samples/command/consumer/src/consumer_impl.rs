// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::{info, warn};
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
        warn!("Got a publish request: {request:?}");

        Err(Status::unimplemented("publish has not been implemented"))
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
    use async_std::task;
    use uuid::Uuid;

    #[test]
    fn respond_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-id");
        let response_id = Uuid::new_v4().to_string();
        let payload = String::from("some-payload");

        let request = tonic::Request::new(RespondRequest { entity_id, response_id, payload });
        let result = task::block_on(consumer_impl.respond(request));
        assert!(result.is_ok());
    }
}
