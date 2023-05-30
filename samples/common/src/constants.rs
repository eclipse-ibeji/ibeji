// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

/// Supported digital twin operations.
pub mod digital_twin_operation {
    pub const GET: &str = "Get";
    pub const SET: &str = "Set";
    pub const SUBSCRIBE: &str = "Subscribe";
    pub const UNSUBSCRIBE: &str = "Unsubscribe";
    pub const INVOKE: &str = "Invoke";
}

// Supported gitial twin protocols.
pub mod digital_twin_protocol {
    pub const GRPC: &str = "grpc";
}
