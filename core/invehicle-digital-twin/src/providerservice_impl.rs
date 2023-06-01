// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core_protobuf_data_access::chariott::{
    common::v1::{
        discover_fulfillment::Service, DiscoverFulfillment, fulfillment::Fulfillment as FulfillmentEnum,
        Fulfillment as FulfillmentMessage, intent::Intent as IntentEnum,
    },
    provider::v1::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};
use log::info;
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use url::Url;

pub const CHARIOTT_SCHEMA_KIND_FOR_GRPC: &str = "grpc+proto";
pub const CHARIOTT_SCHEMA_REFERENCE_FOR_DIGITAL_TWIN_SERVICE : &str = "digital_twin.v1";

#[derive(Debug)]
pub struct ProviderServiceImpl {
    pub url: Url,
}

#[tonic::async_trait]
impl ProviderService for ProviderServiceImpl {
    /// Fulfill implementation.
    ///
    /// # Arguments
    /// * `request` - Fulfill request.
    async fn fulfill(
        &self,
        request: Request<FulfillRequest>,
    ) -> Result<Response<FulfillResponse>, Status> {
        info!("Received a fulfill request");
        let fulfillment = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified."))?
        {
            IntentEnum::Discover(_intent) => Ok(FulfillmentEnum::Discover(DiscoverFulfillment {
                services: vec![Service {
                    url: self.url.to_string(),
                    schema_kind: CHARIOTT_SCHEMA_KIND_FOR_GRPC.to_owned(),
                    schema_reference: CHARIOTT_SCHEMA_REFERENCE_FOR_DIGITAL_TWIN_SERVICE.to_owned(),
                    metadata: HashMap::new(),
                }],
            })),
            _ => Err(Status::unknown("Unsupported or unknown intent."))?,
        };

        fulfillment.map(|f| {
            Response::new(FulfillResponse {
                fulfillment: Some(FulfillmentMessage { fulfillment: Some(f) }),
            })
        })
    }    
}

impl ProviderServiceImpl {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

#[cfg(test)]
mod providerservice_impl_tests {
    use super::*;
    use core_protobuf_data_access::chariott::common::v1::{
        DiscoverIntent, Intent as IntentMessage
    };

    #[tokio::test]
    async fn fulfill_test() {
        let provider_service_impl =
            ProviderServiceImpl { url: Url::parse("http://0.0.0.0:80").unwrap() };

        let request = Request::new(FulfillRequest {
            intent: Some(IntentMessage {
                intent: Some(IntentEnum::Discover(DiscoverIntent {})),
            }),
        });        
        let result = provider_service_impl.fulfill(request).await;
        assert!(result.is_ok(), "fulfill result is not okay: {result:?}");

        let response = result.unwrap();
        let response_inner = response.into_inner();
        assert!(response_inner.fulfillment.is_some());
    }
}
