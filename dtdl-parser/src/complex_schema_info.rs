// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::schema_info::SchemaInfo;

/// A complex schema is the base trait for all complex schemas, like object, map and array.
pub trait ComplexSchemaInfo: SchemaInfo {}
