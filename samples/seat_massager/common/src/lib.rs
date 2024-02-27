// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetedPayload {
    #[serde(rename = "@type")]
    pub model_id: String,
    #[serde(rename = "@id")]    
    pub instance_id: String,
    pub member_name: String,
    pub payload: String
}
