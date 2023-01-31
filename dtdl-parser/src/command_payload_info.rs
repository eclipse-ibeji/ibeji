// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::schema_field_info::SchemaFieldInfo;

/// A command payload specifies the inputs and outputs for a command.
pub trait CommandPayloadInfo: SchemaFieldInfo {}
