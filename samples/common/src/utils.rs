// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::constants;

use log::{debug, info};
use samples_protobuf_data_access::chariott::runtime::v1::chariott_service_client::ChariottServiceClient;
use samples_protobuf_data_access::chariott::{
    common::v1::{
        discover_fulfillment, fulfillment::Fulfillment as FulfillmentEnum,
        intent::Intent as IntentEnum, DiscoverIntent, Intent as IntentMessage,
    },
    runtime::v1::FulfillRequest,
};
use samples_protobuf_data_access::digital_twin::v1::digital_twin_client::DigitalTwinClient;
use samples_protobuf_data_access::digital_twin::v1::{EndpointInfo, FindByIdRequest};
use std::future::Future;
use tokio::time::{sleep, Duration};
use tonic::{Code, Request, Status};

/// Is the provided subset a subset of the provided superset?
///
/// # Arguments
/// * `subset` - The provided subset.
/// * `superset` - The provided superset.
pub fn is_subset(subset: &[String], superset: &[String]) -> bool {
    subset.iter().all(|subset_member| {
        superset.iter().any(|supserset_member| subset_member == supserset_member)
    })
}

///
/// Retry an async function that uses tonic::Status in for its error result.
///
/// # Arguments
/// * `max_retries` - The maximum number of retries.
/// * `duration_between_attempts` - The duration of time attempts.
/// * `function` - The function.
pub async fn retry_async_based_on_status<T, Fut, F: FnMut() -> Fut>(
    max_retries: i32,
    duration_between_attempts: Duration,
    mut function: F,
) -> Result<T, Status>
where
    Fut: Future<Output = Result<T, Status>>,
{
    let mut last_status;
    let mut retries: i32 = 0;

    loop {
        match function().await {
            Ok(t) => return Ok(t),
            Err(status) => {
                if status.code() == Code::Unavailable || status.code() == Code::Internal {
                    last_status = status;
                } else {
                    return Err(status);
                }
            }
        }
        if retries < max_retries {
            debug!("Retrying a call.");
            sleep(duration_between_attempts).await;
            retries += 1;
        } else {
            break;
        }
    }

    Err(last_status)
}

/// Use Ibeji to discover the endpoint for a digital twin provider that satifies the requirements.
///
/// # Arguments
/// * `invehicle_digitial_twin_servuce_url` - In-vehicle digital twin service URL.
/// * `entity_id` - The matching entity id.
/// * `protocol` - The required protocol.
/// * `operations` - The required operations.
pub async fn discover_digital_twin_provider_using_ibeji(
    invehicle_digitial_twin_servuce_url: &str,
    entity_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<EndpointInfo, String> {
    info!("Sending a find_by_id request for entity id {entity_id} to the In-Vehicle Digital Twin Service URL {invehicle_digitial_twin_servuce_url}");
    let mut client = DigitalTwinClient::connect(invehicle_digitial_twin_servuce_url.to_string())
        .await
        .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByIdRequest { id: entity_id.to_string() });
    let response = client.find_by_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    let entity_access_info = response_inner.entity_access_info.expect("Did not find the entity");

    match entity_access_info.endpoint_info_list.iter()
        .find(|endpoint_info| endpoint_info.protocol == protocol && is_subset(operations, endpoint_info.operations.as_slice())).cloned()
    {
        Some(result) => {
            info!("Found a matching endpoint for entity id {entity_id} that has URI {}", result.uri);
            Ok(result)
        },
        None => Err("Did not find an endpoint that met our requirements".to_string())
    }
}

/// Use Chariott to discover the endpoint for the digital twin service.
///
/// # Arguments
/// * `chariott_url` - Chariott's URL.
pub async fn discover_digital_twin_service_using_chariott(
    chariott_url: &str,
) -> Result<String, Status> {
    let mut client = ChariottServiceClient::connect(chariott_url.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let request = Request::new(FulfillRequest {
        namespace: constants::chariott::NAMESPACE_FOR_IBEJI.to_string(),
        intent: Some(IntentMessage { intent: Some(IntentEnum::Discover(DiscoverIntent {})) }),
    });

    // Get list of services at the requested namespace, if any.
    let services: Option<Vec<discover_fulfillment::Service>> = client
        .fulfill(request)
        .await?
        .into_inner()
        .fulfillment
        .and_then(|fulfillment_message| fulfillment_message.fulfillment)
        .and_then(|fulfillment_enum| match fulfillment_enum {
            FulfillmentEnum::Discover(discover) => Some(discover.services.into_iter().collect()),
            _ => None,
        });

    // If we discovered one or more service, then return the URL for the first one that uses gRPC.
    if services.is_some() {
        for service in services.unwrap() {
            if service.schema_kind == constants::chariott::SCHEMA_KIND_FOR_GRPC {
                return Ok(service.url);
            }
        }
        Err(Status::not_found("Failed to discover the in-vehicle digital twin service's URL, as none of the services found had the '{constants::chariott::SCHEMA_KIND_FOR_GRPC}' schema kind"))
    } else {
        Err(Status::not_found("Failed to discover the in-vehicle digital twin service's URL, as it is not registered with Chariott"))
    }
}

/// Retrieve the In-Vehicle Digital Twin URL.
/// If invehicle_digital_twin_url is provided, then it's value is returned.
/// Otherwise, chariott_url is used to retrieve it from Chariott.
///
/// # Arguments
/// * `invehicle_digital_twin_url` - Optional, In-Vehicle Digital Twin URL.
/// * `chariott_url` - Optional, Chariott URL.
pub async fn retrieve_invehicle_digital_twin_url(
    invehicle_digital_twin_url: Option<String>,
    chariott_url: Option<String>,
) -> Result<String, String> {
    // Get the URL for the In-Vehicle Digital Twin Service.
    // First try to use the one specified in the invehicle_digital_twin_url setting.
    // If it is not set, then go to Chariott to obtain it.
    let result = match invehicle_digital_twin_url {
        Some(value) => value,
        None => {
            match chariott_url {
                Some(value) => {
                    match retry_async_based_on_status(30, Duration::from_secs(1), || discover_digital_twin_service_using_chariott(&value)).await {
                        Ok(value) => value,
                        Err(error) => Err(format!("Failed to discover the in-vehicle digital twin service's URL due to error: {error}"))?
                    }
                }
                None => {
                    Err("The settings file must set a chariott_url setting when the invehicle_digital_twin_url is not set.")?
                }
            }
        }
    };

    Ok(result)
}

#[cfg(test)]
mod ibeji_common_utils_tests {
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
