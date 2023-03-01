// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::schema_field_info::SchemaFieldInfo;

/// A command payload specifies the inputs and outputs for a command.
pub trait CommandPayloadInfo: SchemaFieldInfo {}
