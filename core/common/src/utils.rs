// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::debug;
use std::future::Future;
use tokio::time::{sleep, Duration};
use tonic::{Code, Status};

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
