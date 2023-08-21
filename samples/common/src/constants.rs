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
    pub const STREAM: &str = "Stream";
}

// Supported digital twin protocols.
pub mod digital_twin_protocol {
    pub const GRPC: &str = "grpc";
    pub const MQTT: &str = "mqtt";
}

pub mod chariott {
    // pub const SCHEMA_KIND_FOR_GRPC: &str = "grpc+proto";
    pub const INVEHICLE_DIGITAL_TWIN_SERVICE_NAMESPACE: &str = "sdv.ibeji";
    pub const INVEHICLE_DIGITAL_TWIN_SERVICE_NAME: &str = "invehicle_digital_twin";
    pub const INVEHICLE_DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
    pub const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_KIND: &str = "grpc+proto";
    pub const INVEHICLE_DIGITAL_TWIN_SERVICE_COMMUNICATION_REFERENCE: &str = "https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/digital_twin/v1/digital_twin.proto";
}
