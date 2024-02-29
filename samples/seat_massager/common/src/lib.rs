// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TargetedPayload {
    #[serde(rename = "@id")]
    pub instance_id: String,
    pub operation: String,
    pub payload: String,
}
