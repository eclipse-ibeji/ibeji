// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use config::{Config, File, FileFormat};
use log::debug;
use std::future::Future;
use tokio::time::{sleep, Duration};

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
