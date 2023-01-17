// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use crate::complex_schema_info::ComplexSchemaInfo;
use crate::entity_info::EntityInfo;
use crate::field_info::FieldInfo;

pub trait ObjectInfo : ComplexSchemaInfo {
    fn as_entity_info(&self) -> &dyn EntityInfo;

    // TODO: should this be optional?
    fn fields(&self) -> &Vec<Box<dyn FieldInfo>>;

    // fn add_field(&mut self, name: String, id: Dtmi, parent_id: Option<Dtmi>, schema: Option<Box<dyn SchemaInfo>>);
}