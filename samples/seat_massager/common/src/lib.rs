// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TargetedPayload {
    pub instance_id: String,
    pub member_path: String,
    pub operation: String,
    pub payload: String,
}
