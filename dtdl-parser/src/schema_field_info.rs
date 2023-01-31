// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::named_entity_info::NamedEntityInfo;
use crate::schema_info::SchemaInfo;

/// An abstract trait that represents an entity that has a schema field.
pub trait SchemaFieldInfo: NamedEntityInfo {
    /// Returns the schema.
    fn schema(&self) -> &Option<Box<dyn SchemaInfo>>;
}
