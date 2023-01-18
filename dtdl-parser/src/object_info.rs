// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::complex_schema_info::ComplexSchemaInfo;
use crate::field_info::FieldInfo;

pub trait ObjectInfo : ComplexSchemaInfo {
    // TODO: should this be optional?
    /// Returns the fields.
    fn fields(&self) -> &Vec<Box<dyn FieldInfo>>;
}