// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use config::{Config, File, FileFormat};
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use log::{debug, info};
use serde_derive::Deserialize;
use std::future::Future;
use tokio::time::{sleep, Duration};
use tonic::{Request, Status};

#[derive(Debug, Deserialize)]
pub struct ServiceIdentifier {
    pub namespace: String,
    pub name: String,
    pub version: String,
}

/// Load the settings.
///
/// # Arguments
/// * `config_filename` - Name of the config file to load settings from.
pub fn load_settings<T>(config_filename: &str) -> T
where
    T: for<'de> serde::Deserialize<'de>,
{
    let config =
        Config::builder().add_source(File::new(config_filename, FileFormat::Yaml)).build().unwrap();

    let settings: T = config.try_deserialize().unwrap();

    settings
}

/// Retry a function that returns an error.
///
/// # Arguments
/// * `max_retries` - The maximum number of retries.
/// * `retry_interval_ms` - The retry interval between retries in milliseconds.
/// * `function` - The function to retry.
/// * `context` - Context field to provide additional info for logging.
pub async fn execute_with_retry<T, E, Fut, F: FnMut() -> Fut>(
    max_retries: u32,
    retry_interval_ms: Duration,
    mut function: F,
    context: Option<String>,
) -> Result<T, E>
where
    Fut: Future<Output = Result<T, E>>,
{
    let mut last_error: Result<T, E>;
    let mut retries = 0;

    loop {
        match function().await {
            Ok(t) => return Ok(t),
            Err(error) => {
                last_error = Err(error);
            }
        }
        debug!(
            "Retrying the function call. Total retry attempts: {retries} (context: {context:?})"
        );

        sleep(retry_interval_ms).await;

        retries += 1;

        if retries == max_retries {
            break;
        }
    }
    last_error
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

    let response = client.discover(request).await?;

    let service = response.into_inner().service.ok_or_else(|| Status::not_found("Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version}"))?;

    if service.communication_kind != communication_kind
        && service.communication_reference != communication_reference
    {
        return Err(Status::not_found(
            "Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version} that has communication kind '{communication_kind} and communication_reference '{communication_reference}''",
        ));
    }

    Ok(service.uri)
}

/// Get a service's URI from settings or from Chariott.
/// Will first try to use the URI defined in the service's settings file. If that is not set, will
/// call Chariott to obtain it.
///
/// # Arguments
/// * `service_uri` - Optional, desired service's URI.
/// * `chariott_uri` - Optional, Chariott's URI.
/// * `service_identifier` - Optional, The service's identifiers (name, namespace, version).
/// # `communication_kind` - Optional, The service's communication kind.
/// # `communication_reference` - Optional, The service's communication reference.
pub async fn get_service_uri(
    service_uri: Option<String>,
    chariott_uri: Option<String>,
    service_identifier: Option<ServiceIdentifier>,
    communication_kind: &str,
    communication_reference: &str,
) -> Result<String, Status> {
    let result = match service_uri {
        Some(value) => {
            info!("URI set in settings.");
            value
        }
        None => match chariott_uri {
            Some(value) => {
                info!("Retrieving URI from Chariott.");

                let service_identifier = service_identifier.ok_or_else(|| Status::invalid_argument("The settings file must set the service_identifier when the chariott_uri is set."))?;

                execute_with_retry(
                    30,
                    Duration::from_secs(1),
                    || {
                        discover_service_using_chariott(
                            &value,
                            &service_identifier.namespace,
                            &service_identifier.name,
                            &service_identifier.version,
                            communication_kind,
                            communication_reference,
                        )
                    },
                    Some(format!(
                        "Attempting to discover service '{}' with chariott.",
                        service_identifier.name
                    )),
                )
                .await?
            }
            None => Err(Status::invalid_argument(
                "The settings file must set the chariott_uri when the service_uri is not set.",
            ))?,
        },
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    async fn test_function(attempts: Rc<RefCell<u32>>) -> Result<(), ()> {
        let mut attempts = attempts.borrow_mut();
        *attempts += 1;
        if *attempts == 3 {
            Ok(())
        } else {
            Err(())
        }
    }

    #[tokio::test]
    async fn test_retry_async_function() {
        const MAX_RETRIES: u32 = 3;

        let attempts = Rc::new(RefCell::new(0));
        let mut result = execute_with_retry(
            MAX_RETRIES,
            Duration::from_secs(1),
            || test_function(attempts.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(*attempts.borrow(), MAX_RETRIES);

        *attempts.borrow_mut() = 4;
        result = execute_with_retry(
            MAX_RETRIES,
            Duration::from_secs(1),
            || test_function(attempts.clone()),
            Some(String::from("test_retry_context")),
        )
        .await;
        assert!(result.is_err());
    }
}
