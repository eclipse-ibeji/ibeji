// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::complex_schema_info::ComplexSchemaInfo;
use crate::field_info::FieldInfo;

pub trait ObjectInfo : ComplexSchemaInfo {
    /// Returns the fields.
    fn fields(&self) -> &Option<Vec<Box<dyn FieldInfo>>>;
}