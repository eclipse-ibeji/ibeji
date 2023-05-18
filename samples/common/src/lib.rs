// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::{debug, info};
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, FindByIdRequest};

/// Supported digital twin operations.
pub mod digital_twin_operation {
    pub const GET: &str = "Get";
    pub const SET: &str = "Set";
    pub const SUBSCRIBE: &str = "Subscribe";
    pub const UNSUBSCRIBE: &str = "Unsubscribe";
    pub const INVOKE: &str = "Invoke";
}

// Supported gitial twin protocols.
pub mod digital_twin_protocol {
    pub const GRPC: &str = "grpc";
}

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

/// Find a provider endpoint that satifies the requirements.
///
/// # Arguments
/// `in_vehcile_digitial_twin_servuce_uri` - iI-vehicle digital twin service URI.
/// `entity_id` - The matching entity id.
/// `protocol` - The required protocol.
/// `operations` - The required operations.
pub async fn find_provider_endpoint(
    in_vehicle_digitial_twin_servuce_uri: &'static str,
    entity_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<EndpointInfo, String> {
    info!("Sending a find_by_id request for entity id {entity_id} to the In-Vehicle Digital Twin Service URI {in_vehicle_digitial_twin_servuce_uri}");
    let mut client = DigitalTwinClient::connect(in_vehicle_digitial_twin_servuce_uri)
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

#[cfg(test)]
mod ibeji_common_tests {
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
