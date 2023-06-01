// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::{debug, info};
use samples_protobuf_data_access::chariott::{
    common::v1::{
        discover_fulfillment,
        DiscoverIntent,
        fulfillment::Fulfillment as FulfillmentEnum,
        intent::Intent as IntentEnum,
        Intent as IntentMessage,        
    },
    runtime::v1::{FulfillRequest},
};
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, FindByIdRequest};
use samples_protobuf_data_access::chariott::runtime::v1::chariott_service_client::ChariottServiceClient;
use tonic::{Request, Status};

pub const CHARIOTT_NAMESPACE_FOR_IBEJI: &str = "sdv.ibeji";
pub const CHARIOTT_SCHEMA_KIND_FOR_GRPC: &str = "grpc+proto";

/// Is the provided subset a subset of the provided superset?
///
/// # Arguments
/// `subset` - The provided subset.
/// `superset` - The provided superset.
pub fn is_subset(subset: &[String], superset: &[String]) -> bool {
    subset.iter().all(|subset_member| {
        superset.iter().any(|supserset_member| subset_member == supserset_member)
    })
}

/// Use Ibeji to discover the endpoint for a digital twin provider that satifies the requirements.
///
/// # Arguments
/// `invehcile_digitial_twin_servuce_url` - In-vehicle digital twin service URL.
/// `entity_id` - The matching entity id.
/// `protocol` - The required protocol.
/// `operations` - The required operations.
pub async fn discover_digital_twin_provider_using_ibeji(
    invehicle_digitial_twin_servuce_url: &'static str,    
    entity_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<EndpointInfo, String> {
    info!("Sending a find_by_id request for entity id {entity_id} to the In-Vehicle Digital Twin Service URL {invehicle_digitial_twin_servuce_url}");
    let mut client = DigitalTwinClient::connect(invehicle_digitial_twin_servuce_url)
        .await
        .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByIdRequest { id: entity_id.to_string() });
    let response = client.find_by_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    let entity_access_info = response_inner.entity_access_info.expect("Did not find the entity");

    let mut matching_endpoint_info_option: Option<EndpointInfo> = None;
    for endpoint_info in entity_access_info.endpoint_info_list {
        // We require and endpoint that supports the protocol and supports all of the operations.
        if endpoint_info.protocol == protocol
            && is_subset(operations, endpoint_info.operations.as_slice())
        {
            matching_endpoint_info_option = Some(endpoint_info);
            break;
        }
    }

    if matching_endpoint_info_option.is_none() {
        return Err("Did not find an endpoint that met our requirements".to_string());
    }

    let result = matching_endpoint_info_option.unwrap();

    info!("Found a matching endpoint for entity id {entity_id} that has URI {}", result.uri);

    Ok(result)
}

/// Use Chariott to discover the endpoint for the digital twin service.
///
/// # Arguments
/// * `chariott_url` - Chariott's URL.
pub async fn discover_digital_twin_service_using_chariott(chariott_url: &str) -> Result<Option<String>, Status> {
    let mut client = ChariottServiceClient::connect(chariott_url.to_string()).await.map_err(|e|Status::internal(e.to_string()))?;

    let request = Request::new(FulfillRequest {
        namespace: CHARIOTT_NAMESPACE_FOR_IBEJI.to_string(),
        intent: Some(IntentMessage {
            intent: Some(IntentEnum::Discover(DiscoverIntent {})),
        }),
    });

    // Get list of services at the requested namespace, if any.
    let services: Option<Vec<discover_fulfillment::Service>> = client
        .fulfill(request)
        .await?
        .into_inner()
        .fulfillment
        .and_then(|fulfillment_message| fulfillment_message.fulfillment)
        .and_then(|fulfillment_enum| match fulfillment_enum {
            FulfillmentEnum::Discover(discover) => {
                Some(discover.services.into_iter().collect())
            }
            _ => None,
        });

    // If we discovered one or more service, then return the URL for the first one that uses gRPC.
    if services.is_some() {
        for service in services.unwrap() {
            if service.schema_kind == CHARIOTT_SCHEMA_KIND_FOR_GRPC {
                return Ok(Some(service.url.to_string()))
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod ibeji_common_misc_tests {
    use super::*;

    #[test]
    fn is_subset_test() {
        assert!(is_subset(&[], &[]));
        assert!(is_subset(&[], &["one".to_string()]));
        assert!(is_subset(&[], &["one".to_string(), "two".to_string()]));
        assert!(is_subset(&["one".to_string()], &["one".to_string(), "two".to_string()]));
        assert!(is_subset(
            &["one".to_string(), "two".to_string()],
            &["one".to_string(), "two".to_string()]
        ));
        assert!(!is_subset(
            &["one".to_string(), "two".to_string(), "three".to_string()],
            &["one".to_string(), "two".to_string()]
        ));
        assert!(!is_subset(
            &["one".to_string(), "two".to_string(), "three".to_string()],
            &["one".to_string()]
        ));
        assert!(!is_subset(&["one".to_string(), "two".to_string(), "three".to_string()], &[]));
    }
}
