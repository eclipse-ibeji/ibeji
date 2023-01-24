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
        let request_inner = request.into_inner();

        info!(
            "Received a publish for entity id {} with the value {}",
            request_inner.entity_id, request_inner.value
        );

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
        warn!("Got a respons request: {:?}", request);

        Err(Status::unimplemented("set has not been implemented"))
    }
}

#[cfg(test)]
mod consumer_impl_tests {
    use super::*;
    use async_std::task;

    #[test]
    fn publish_test() {
        let consumer_impl = ConsumerImpl {};

        let entity_id = String::from("some-id");
        let value = String::from("some-value");

        let request = tonic::Request::new(PublishRequest { entity_id, value });
        let result = task::block_on(consumer_impl.publish(request));
        assert!(result.is_ok());
    }
}
