// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::constants;

use constants::chariott::{
    INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND,
    INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE, INVEHICLE_DIGITAL_TWIN_SERVICE_NAME,
    INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE, INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION,
};
use log::{debug, info};
use samples_protobuf_data_access::chariott::service_discovery::core::v1::service_registry_client::ServiceRegistryClient;
use samples_protobuf_data_access::chariott::service_discovery::core::v1::DiscoverRequest;
use samples_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_client::InvehicleDigitalTwinClient;
use samples_protobuf_data_access::invehicle_digital_twin::v1::{EndpointInfo, FindByIdRequest};
use std::future::Future;
use tokio::time::{sleep, Duration};
use tonic::{Code, Request, Status};

/// Is the provided subset a subset of the provided superset?
///
/// # Arguments
/// * `subset` - The provided subset.
/// * `superset` - The provided superset.
fn is_subset(subset: &[String], superset: &[String]) -> bool {
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
/// * `invehicle_digitial_twin_service_uri` - In-vehicle digital twin service URI.
/// * `entity_id` - The matching entity id.
/// * `protocol` - The required protocol.
/// * `operations` - The required operations.
pub async fn discover_digital_twin_provider_using_ibeji(
    invehicle_digitial_twin_service_uri: &str,
    entity_id: &str,
    protocol: &str,
    operations: &[String],
) -> Result<EndpointInfo, String> {
    info!("Sending a find_by_id request for entity id {entity_id} to the In-Vehicle Digital Twin Service URI {invehicle_digitial_twin_service_uri}");
    let mut client =
        InvehicleDigitalTwinClient::connect(invehicle_digitial_twin_service_uri.to_string())
            .await
            .map_err(|error| format!("{error}"))?;
    let request = tonic::Request::new(FindByIdRequest { id: entity_id.to_string() });
    let response = client.find_by_id(request).await.map_err(|error| error.to_string())?;
    let response_inner = response.into_inner();
    debug!("Received the response for the find_by_id request");
    info!("response_payload: {:?}", response_inner.entity_access_info);

    match response_inner
        .entity_access_info
        .ok_or("Did not find the entity".to_string())?
        .endpoint_info_list
        .iter()
        .find(|endpoint_info| {
            endpoint_info.protocol == protocol
                && is_subset(operations, endpoint_info.operations.as_slice())
        })
        .cloned()
    {
        Some(result) => {
            info!(
                "Found a matching endpoint for entity id {entity_id} that has URI {}",
                result.uri
            );
            Ok(result)
        }
        None => Err("Did not find an endpoint that met our requirements".to_string()),
    }
}

/// Use Chariott to discover a service.
///
/// # Arguments
/// * `chariott_uri` - Chariott's URI.
/// * `namespace` - The service's namespace.
/// * `name` - The service's name.
/// * `version` - The service's version.
/// # `communication_kind` - The service's communication kind.
/// # `communication_reference` - The service's communication reference.
pub async fn discover_service_using_chariott(
    chariott_uri: &str,
    namespace: &str,
    name: &str,
    version: &str,
    communication_kind: &str,
    communication_reference: &str,
) -> Result<String, Status> {
    let mut client = ServiceRegistryClient::connect(chariott_uri.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let request = Request::new(DiscoverRequest {
        namespace: namespace.to_string(),
        name: name.to_string(),
        version: version.to_string(),
    });

    let response =
        client.discover(request).await.map_err(|error| Status::internal(error.to_string()))?;
    let response_inner = response.into_inner();

    let service = match response_inner.service {
        Some(value) => value,
        None => {
            return Err(Status::not_found("Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version}"));
        }
    };

    if service.communication_kind != communication_kind
        && service.communication_reference != communication_reference
    {
        return Err(Status::not_found(
            "Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version} that has communication kind '{communication_kind} and communication_reference '{communication_reference}''",
        ));
    }

    Ok(service.uri)
}

/// Retrieve the In-Vehicle Digital Twin URI.
/// If invehicle_digital_twin_uri is provided, then its value is returned.
/// Otherwise, chariott_uri is used to retrieve it from Chariott.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - Optional, In-Vehicle Digital Twin URI.
/// * `chariott_uri` - Optional, Chariott URI.
pub async fn retrieve_invehicle_digital_twin_uri(
    invehicle_digital_twin_uri: Option<String>,
    chariott_uri: Option<String>,
) -> Result<String, String> {
    // Get the URI for the In-Vehicle Digital Twin Service.
    // First try to use the one specified in the invehicle_digital_twin_uri setting.
    // If it is not set, then go to Chariott to obtain it.
    let result = match invehicle_digital_twin_uri {
        Some(value) => {
            info!("The URI for the in-vehicle digital twin service is specified in the settings file.");
            value
        },
        None => {
            match chariott_uri {
                Some(value) => {
                    info!("The URI for the in-vehicle digital twin service will be retrieved from Chariott.");
                    match retry_async_based_on_status(
                        30,
                        Duration::from_secs(1),
                        || discover_service_using_chariott(
                            &value,
                            INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE,
                            INVEHICLE_DIGITAL_TWIN_SERVICE_NAME,
                            INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION,
                            INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND,
                            INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE)
                    ).await {
                        Ok(value) => value,
                        Err(error) => Err(format!("Failed to discover the in-vehicle digital twin service's URI due to error: {error}"))?
                    }
                }
                None => {
                    Err("The settings file must set a chariott_uri setting when the invehicle_digital_twin_uri is not set.")?
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
