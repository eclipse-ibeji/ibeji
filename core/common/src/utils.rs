// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use bytes::{Bytes, BytesMut};
use config::{Config, ConfigError, File, FileFormat};
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use http_body::Body;
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use log::{debug, info};
use serde_derive::Deserialize;
use std::env;
use std::future::Future;
use strum_macros::Display;
use tokio::time::{sleep, Duration};
use tonic::{Request, Status};

const IBEJI_HOME_VAR_NAME: &str = "IBEJI_HOME";

/// An identifier used when discovering a service through Chariott.
#[derive(Debug, Deserialize)]
pub struct ServiceIdentifier {
    /// The namespace of the service.
    pub namespace: String,
    /// The name of the service.
    pub name: String,
    /// The version of the service.
    pub version: String,
}

/// An enum representing where to discover a service's URI.
#[derive(Display, Debug, Deserialize)]
pub enum ServiceUriSource {
    /// Use the local configuration settings to find the service's URI.
    Local { service_uri: String },
    /// Use Chariott to discover the service's URI.
    Chariott { chariott_uri: String, service_identifier: ServiceIdentifier },
}

/// Load the settings.
///
/// # Arguments
/// * `config_filename` - Name of the config file to load settings from.
pub fn load_settings<T>(config_filename: &str) -> Result<T, ConfigError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let config_filename_path = match std::env::var(IBEJI_HOME_VAR_NAME) {
        Ok(s) => format!("{}/{}", s, config_filename),
        _ => config_filename.to_owned(),
    };

    let config = Config::builder()
        .add_source(File::new(config_filename_path.as_str(), FileFormat::Yaml))
        .build()?;

    config.try_deserialize()
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
/// # `expected_communication_kind` - The service's expected communication kind.
/// # `expected_communication_reference` - The service's expected communication reference.
pub async fn discover_service_using_chariott(
    chariott_uri: &str,
    namespace: &str,
    name: &str,
    version: &str,
    expected_communication_kind: &str,
    expected_communication_reference: &str,
) -> Result<String, Status> {
    let uri = get_uri(chariott_uri)?;

    let mut client =
        ServiceRegistryClient::connect(uri).await.map_err(|e| Status::internal(e.to_string()))?;

    let request = Request::new(DiscoverRequest {
        namespace: namespace.to_string(),
        name: name.to_string(),
        version: version.to_string(),
    });

    let response = client.discover(request).await?;

    let service = response.into_inner().service.ok_or_else(|| Status::not_found("Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version}"))?;

    if service.communication_kind != expected_communication_kind
        && service.communication_reference != expected_communication_reference
    {
        Err(Status::not_found(
            "Did not find a service in Chariott with namespace '{namespace}', name '{name}' and version {version} that has communication kind '{communication_kind} and communication_reference '{communication_reference}''",
        ))
    } else {
        Ok(service.uri)
    }
}

/// Get a service's URI from settings or from Chariott.
///
/// # Arguments
/// * `service_uri_source` - Enum providing information on how to get the service URI.
/// # `expected_communication_kind` - The service's expected communication kind.
/// # `expected_communication_reference` - The service's expected communication reference.
pub async fn get_service_uri(
    service_uri_source: ServiceUriSource,
    expected_communication_kind: &str,
    expected_communication_reference: &str,
) -> Result<String, Status> {
    let result = match service_uri_source {
        ServiceUriSource::Local { service_uri } => {
            info!("URI set in settings.");
            service_uri
        }
        ServiceUriSource::Chariott { chariott_uri, service_identifier } => {
            info!("Retrieving URI from Chariott.");

            execute_with_retry(
                30,
                Duration::from_secs(1),
                || {
                    discover_service_using_chariott(
                        &chariott_uri,
                        &service_identifier.namespace,
                        &service_identifier.name,
                        &service_identifier.version,
                        expected_communication_kind,
                        expected_communication_reference,
                    )
                },
                Some(format!(
                    "Attempting to discover service '{}' with chariott.",
                    service_identifier.name
                )),
            )
            .await?
        }
    };

    let uri = get_uri(&result)?;

    Ok(uri)
}

/// If feature 'containerize' is set, will modify a localhost uri to point to container's localhost
/// DNS alias. Otherwise, returns the uri as a String.
///
/// # Arguments
/// * `uri` - The uri to potentially modify.
pub fn get_uri(uri: &str) -> Result<String, Status> {
    #[cfg(feature = "containerize")]
    let uri = {
        // Container env variable names.
        let host_gateway_env_var: &str = "HOST_GATEWAY";
        let host_alias_env_var: &str = "LOCALHOST_ALIAS";

        // Return an error if container env variables are not set.
        let host_gateway = env::var(host_gateway_env_var).map_err(|err| {
            Status::failed_precondition(format!(
                "Unable to get environment var '{host_gateway_env_var}' with error: {err}"
            ))
        })?;
        let host_alias = env::var(host_alias_env_var).map_err(|err| {
            Status::failed_precondition(format!(
                "Unable to get environment var '{host_alias_env_var}' with error: {err}"
            ))
        })?;

        uri.replace(&host_alias, &host_gateway)
    };

    Ok(uri.to_string())
}

/// Converts an HTTP body to bytes, propagating errors from the body.
/// 
/// # Arguments
/// - `body`: the body to read
/// - `max_length`: an optional maximum number of bytes to read. Body frames will be read until this value is exceeded. Setting this value can help avoid DoS attacks.
// pub async fn to_bytes<'a, T, E>(body: &mut T, max_length: Option<usize>) -> Result<Bytes, E>
// where
//     T: Body<Data = Bytes, Error = E> + Unpin
pub async fn to_bytes(body: &mut UnsyncBoxBody<Bytes, Status>, max_length: Option<usize>) -> Result<Bytes, Status>
{
    let mut buf = BytesMut::new();
    while let Some(next) = body.frame().await {
        let frame = next?;

        // Only capture DATA frames and skip others, such as trailer frames
        if let Some(chunk) = frame.data_ref() {
            buf.extend_from_slice(&chunk[..]);
        }

        if buf.len() >= max_length.unwrap_or(usize::MAX) {
            return Ok(buf.freeze());
        }
    }

    Ok(buf.freeze())
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
