// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::content_info::ContentInfo;
use crate::schema_info::SchemaInfo;

/// A telemetry specifies data that is emitted as a stream.
pub trait TelemetryInfo: ContentInfo {
    /// Returns the schema.
    fn schema(&self) -> &Option<Box<dyn SchemaInfo>>;
}
