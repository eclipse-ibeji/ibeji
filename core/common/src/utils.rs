// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use config::{Config, ConfigError, File, FileFormat};
use core_protobuf_data_access::chariott::service_discovery::core::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use log::{debug, info};
use serde_derive::Deserialize;
use std::{env, thread};
use std::future::Future;
use strum_macros::Display;
use tokio::{time::{sleep, Duration}, runtime::Handle};
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

/// Blocks on an asynchronous task, waiting for it to complete.
/// This can be used to execute async code in a sync context.
/// This will start a new thread using the current tokio runtime.
///
/// # Panics
/// This will panic when called outside of a tokio runtime context.
/// This can also theoretically panic if `std::thread::JoinHandle::join` panics,
/// but this is believed to be impossible in this usage.
///
/// For more information on the panic conditions, see
/// <https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.current> and
/// <https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join>
///
/// # Arguments
/// - `future`: the future to execute
/// - `timeout`: the maximum amount of time that `future` should execute before being cancelled
pub fn block_on<T, E, F>(future: F, timeout: Duration) -> Result<T, BlockOnError<E>>
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    let handle = Handle::current();
    let thread = thread::spawn(move || {
        handle.block_on(async move {
            tokio::time::timeout(timeout, future).await
        })
    });

    match thread.join() {
        Ok(Ok(r)) => r.map_err(|e| BlockOnError::InnerError(e)),
        Ok(Err(_)) => Err(BlockOnError::Timeout),
        Err(e) => Err(BlockOnError::JoinError(format!("{e:?}"))),
    }
}

#[derive(Debug)]
pub enum BlockOnError<E> {
    Timeout,
    JoinError(String),
    InnerError(E),
}

impl<E: std::error::Error + 'static> std::error::Error for BlockOnError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BlockOnError::InnerError(e) => Some(e),
            _ => None,
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for BlockOnError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockOnError::Timeout => write!(f, "Execution timed out"),
            BlockOnError::JoinError(s) => write!(f, "Error joining thread (most likely the future passed to block_on panicked): {s}"),
            BlockOnError::InnerError(e) => write!(f, "Error during execution: {e}"),
        }
    }
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
