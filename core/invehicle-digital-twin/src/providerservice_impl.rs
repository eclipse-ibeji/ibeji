// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// extern crate iref;

// use core_protobuf_data_access::chariott::common::v1::Fulfillment;
/*
use core_protobuf_data_access::chariott::provider::v1::provider_service_server::ProviderService;
use core_protobuf_data_access::chariott::provider::v1::{
    FulfillRequest, FulfillResponse,
};
use log::info;
*/

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
                    schema_kind: "grpc+proto".to_owned(),
                    schema_reference: "example.provider.v1".to_owned(),
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
    // use core_protobuf_data_access::digital_twin::v1::EndpointInfo;

/*
    #[tokio::test]
    async fn find_by_id_test() {
        let operations = vec![String::from("Subscribe"), String::from("Unsubscribe")];

        let endpoint_info = EndpointInfo {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            operations,
        };

        let entity_access_info = EntityAccessInfo {
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoint_info_list: vec![endpoint_info],
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl =
            DigitalTwinImpl { entity_access_info_map: entity_access_info_map.clone() };

        // This block controls the lifetime of the lock.
        {
            let mut lock: RwLockWriteGuard<HashMap<String, EntityAccessInfo>> =
                entity_access_info_map.write();
            lock.insert(entity_access_info.id.clone(), entity_access_info.clone());
        }

        let request = tonic::Request::new(FindByIdRequest {
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
        });
        let result = digital_twin_impl.find_by_id(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let response_inner = response.into_inner();

        assert!(response_inner.entity_access_info.is_some());

        let response_entity_access_info = response_inner.entity_access_info.unwrap();

        assert_eq!(
            response_entity_access_info.id,
            "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"
        );
        assert_eq!(response_entity_access_info.endpoint_info_list.len(), 1);
        assert_eq!(
            response_entity_access_info.endpoint_info_list[0].uri,
            "http://[::1]:40010" // Devskim: ignore DS137138
        );
    }

    #[tokio::test]
    async fn register_test() {
        let endpoint_info = EndpointInfo {
            protocol: String::from("grpc"),
            uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
            context: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            operations: vec![String::from("Subscribe"), String::from("Unsubscribe")],
        };

        let entity_access_info = EntityAccessInfo {
            name: String::from("AmbientAirTemperature"),
            id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
            description: String::from("Ambient air temperature"),
            endpoint_info_list: vec![endpoint_info],
        };

        let entity_access_info_map = Arc::new(RwLock::new(HashMap::new()));

        let digital_twin_impl =
            DigitalTwinImpl { entity_access_info_map: entity_access_info_map.clone() };

        let request = tonic::Request::new(RegisterRequest {
            entity_access_info_list: vec![entity_access_info],
        });
        let result = digital_twin_impl.register(request).await;
        assert!(result.is_ok(), "register result is not okay: {result:?}");

        // This block controls the lifetime of the lock.
        {
            let lock: RwLockReadGuard<HashMap<String, EntityAccessInfo>> =
                entity_access_info_map.read();
            // Make sure that we populated the entity map from the contents of the DTDL.
            assert_eq!(lock.len(), 1, "expected length was 1, actual length is {}", lock.len());
        }
    }
*/
}
