// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;
use crate::entity_kind::EntityKind;
use crate::primitive_schema_info::PrimitiveSchemaInfo;
use crate::schema_info::SchemaInfo;

#[derive(Clone, PartialEq)]
pub struct PrimitiveSchemaInfoImpl {
    // EntitytInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    entity_kind: EntityKind,
    description: Option<String>,
    display_name: Option<String>,
    undefined_properties: HashMap<String, Value>,
}

impl PrimitiveSchemaInfoImpl {
    /// Returns a new PrimitiveSchemaInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - The DTDL version used to define the primitive schema.
    /// * `id` - The identifier.
    /// * `child_of` - The identifier of the parent element in which this primitive schema is defined.
    /// * `defined_in` - The identifier of the partition in which this relationship is defined.
    /// * `entity_kind` - The entity kind.
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        entity_kind: EntityKind,
    ) -> Self {
        Self {
            dtdl_version,
            id,
            child_of,
            defined_in,
            entity_kind,
            description: None,
            display_name: None,            
            undefined_properties: HashMap::<String, Value>::new(),
        }
    }

    /// Add an undefined property.
    /// # Arguments
    /// * `key` - The property's name.
    /// * `value` - The property's value.
    pub fn add_undefined_property(&mut self, key: String, value: Value) {
        self.undefined_properties.insert(key, value);
    }    
}

impl EntityInfo for PrimitiveSchemaInfoImpl {
    /// Returns the DTDL version.
    fn dtdl_version(&self) -> i32 {
        self.dtdl_version
    }

    /// Returns the identifier.
    fn id(&self) -> &Dtmi {
        &self.id
    }

    /// Returns the kind of entity.
    fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }

    /// Returns the parent identifier.
    fn child_of(&self) -> &Option<Dtmi> {
        &self.child_of
    }

    /// Returns the enclosing partition's identifier.
    fn defined_in(&self) -> &Option<Dtmi> {
        &self.defined_in
    }

    // Returns the description for this entity.
    fn description(&self) -> &Option<String> {
        &self.description
    }

    // Returns the display name for this entity.
    fn display_name(&self) -> &Option<String> {
        &self.display_name
    }
    
    /// Returns all undefined properties.
    fn undefined_properties(&self) -> &HashMap<String, Value> {
        &self.undefined_properties
    }

    /// Returns the instance as an Any.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SchemaInfo for PrimitiveSchemaInfoImpl {}

impl PrimitiveSchemaInfo for PrimitiveSchemaInfoImpl {}

#[cfg(test)]
mod primitive_schema_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::model_parser::DTDL_VERSION;
    use serde_json;

    #[test]
    fn new_primitive_schema_info_impl_test() {
        let id_result: Option<Dtmi> = create_dtmi("dtmi:com:example:String;1.0");
        assert!(id_result.is_some());
        let id = id_result.unwrap();

        let first_propery_value: Value = serde_json::from_str("{\"first\": \"this\"}").unwrap();
        let second_propery_value: Value = serde_json::from_str("{\"second\": \"that\"}").unwrap();

        let mut primitive_schema_info =
            PrimitiveSchemaInfoImpl::new(DTDL_VERSION, id.clone(), None, None, EntityKind::String);
        primitive_schema_info
            .add_undefined_property(String::from("first"), first_propery_value.clone());
        primitive_schema_info
            .add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(primitive_schema_info.dtdl_version() == 2);
        assert!(primitive_schema_info.id() == &id);
        assert!(primitive_schema_info.child_of().is_none());
        assert!(primitive_schema_info.defined_in().is_none());
        assert!(primitive_schema_info.entity_kind() == EntityKind::String);
        assert!(primitive_schema_info.undefined_properties().len() == 2);
        assert!(
            primitive_schema_info.undefined_properties().get("first").unwrap().clone()
                == first_propery_value
        );
        assert!(
            primitive_schema_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );
    }
}
