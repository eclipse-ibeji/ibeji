// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde_derive::{Deserialize, Serialize};

/// A targeted payload.
/// The targeting details helps on the receiver's side to dispatch the request.
#[derive(Serialize, Deserialize)]
pub struct TargetedPayload {
    /// The instance id for the target entity.
    pub instance_id: String,
    /// The path within the target enttity to member that we are targetting.
    pub member_path: String,
    /// The operation to be performed on the target entity's member.
    pub operation: String,
    /// The operation's payload.
    pub payload: String,
}


pub mod status {
    pub mod ok {
        pub const CODE: i32 = 200;
        pub const MESSAGE: &str = "Ok";
    }
}