// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::complex_schema_info::ComplexSchemaInfo;
use crate::field_info::FieldInfo;

/// An object specifies a value compromised of named fields.
pub trait ObjectInfo: ComplexSchemaInfo {
    /// Returns the fields.
    fn fields(&self) -> &Option<Vec<Box<dyn FieldInfo>>>;
}
