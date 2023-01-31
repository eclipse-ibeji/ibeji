// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::schema_info::SchemaInfo;

/// A complex schema is the base trait for all complex schemas, like object, map and array.
pub trait ComplexSchemaInfo: SchemaInfo {}
