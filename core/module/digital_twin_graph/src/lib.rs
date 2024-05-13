// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod digital_twin_graph_config;
pub mod digital_twin_graph_impl;
pub mod digital_twin_graph_module;
pub mod respond_impl;

use serde_derive::{Deserialize, Serialize};

/// A targeted payload.
/// The targeting details helps on the receiver's side to dispatch the request.
#[derive(Serialize, Deserialize, Debug)]
pub struct TargetedPayload {
    /// The instance id for the target entity.
    pub instance_id: String,
    /// The path within the target entity to the specific member that we are targeting.
    /// It will be empty when we want to target the entire entity.
    pub member_path: String,
    /// The operation to be performed on the target entity's member.
    pub operation: String,
    /// The operation's payload.
    /// It will be empty when the operation does not require a payload.
    pub payload: String,
}

/// Status codes and messages.
pub mod status {
    pub mod ok {
        pub const CODE: i32 = 200;
        pub const MESSAGE: &str = "Ok";
    }
}

/// Supported digital twin operations.
pub mod digital_twin_operation {
    pub const GET: &str = "Get";
    pub const SET: &str = "Set";
    pub const SUBSCRIBE: &str = "Subscribe";
    pub const UNSUBSCRIBE: &str = "Unsubscribe";
    pub const INVOKE: &str = "Invoke";
    pub const STREAM: &str = "Stream";
    pub const MANAGEDSUBSCRIBE: &str = "ManagedSubscribe";
}

/// Supported digital twin protocols.
pub mod digital_twin_protocol {
    pub const GRPC: &str = "grpc";
    pub const MQTT: &str = "mqtt";
}
