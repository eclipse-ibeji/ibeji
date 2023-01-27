// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::complex_schema_info::ComplexSchemaInfo;
use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;
use crate::entity_kind::EntityKind;
use crate::field_info::FieldInfo;
use crate::object_info::ObjectInfo;
use crate::schema_info::SchemaInfo;

pub struct ObjectInfoImpl {
    // EntitytInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    undefined_properties: HashMap<String, Value>,

    // ObjectInfo
    fields: Option<Vec<Box<dyn FieldInfo>>>,
}

impl ObjectInfoImpl {
    /// Returns a new ObjectInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - The DTDL version of used to define the object.
    /// * `id` - The identifier.
    /// * `child_of` - The identifier of the parent element in which this object is defined.
    /// * `defined_in` - The identifier of the partition in which this object is defined.
    /// * `fields` - The fields.
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        fields: Option<Vec<Box<dyn FieldInfo>>>,
    ) -> Self {
        Self {
            dtdl_version,
            id,
            child_of,
            defined_in,
            undefined_properties: HashMap::<String, Value>::new(),
            fields,
        }
    }
}

impl EntityInfo for ObjectInfoImpl {
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
        EntityKind::Object
    }

    /// Returns the parent's identifier.
    fn child_of(&self) -> &Option<Dtmi> {
        &self.child_of
    }

    /// Returns the enclosing partition's identifier.
    fn defined_in(&self) -> &Option<Dtmi> {
        &self.defined_in
    }

    /// Returns all undefined properties.
    fn undefined_properties(&self) -> &HashMap<String, Value> {
        &self.undefined_properties
    }

    /// Add an undefined property.
    /// # Arguments
    /// * `key` - The property's name.
    /// * `value` - The property's value.
    fn add_undefined_property(&mut self, key: String, value: Value) {
        self.undefined_properties.insert(key, value);
    }

    /// Returns the instance as an Any.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SchemaInfo for ObjectInfoImpl {}

impl ComplexSchemaInfo for ObjectInfoImpl {}

impl ObjectInfo for ObjectInfoImpl {
    // Returns the fields.
    fn fields(&self) -> &Option<Vec<Box<dyn FieldInfo>>> {
        &self.fields
    }
}

#[cfg(test)]
mod object_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::model_parser::DTDL_VERSION;
    use serde_json;

    #[test]
    fn new_object_info_impl_test() -> Result<(), String> {
        let id_result: Option<Dtmi> = create_dtmi("dtmi:com:example:Object;1.0");
        assert!(id_result.is_some());
        let id = id_result.unwrap();

        let child_of_result: Option<Dtmi> = create_dtmi("dtmi:com:example:Cabin;1.0");
        assert!(child_of_result.is_some());
        let child_of = child_of_result.unwrap();

        let defined_in_result: Option<Dtmi> = create_dtmi("dtmi:com:example;1.0");
        assert!(defined_in_result.is_some());
        let defined_in = defined_in_result.unwrap();

        let first_propery_value: Value = serde_json::from_str("{\"first\": \"this\"}").unwrap();
        let second_propery_value: Value = serde_json::from_str("{\"second\": \"that\"}").unwrap();

        let fields = Vec::new();

        let mut object_info = ObjectInfoImpl::new(
            DTDL_VERSION,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            Some(fields),
        );
        object_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        object_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(object_info.dtdl_version() == 2);
        assert!(object_info.id() == &id);
        assert!(object_info.child_of().is_some());
        assert!(object_info.child_of().clone().unwrap() == child_of);
        assert!(object_info.defined_in().is_some());
        assert!(object_info.defined_in().clone().unwrap() == defined_in);
        assert!(object_info.entity_kind() == EntityKind::Object);
        assert!(object_info.undefined_properties().len() == 2);
        assert!(
            object_info.undefined_properties().get("first").unwrap().clone() == first_propery_value
        );
        assert!(
            object_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );

        match object_info.fields() {
            Some(fields) => assert_eq!(fields.len(), 0),
            None => return Err(String::from("fields has not been set")),
        }

        Ok(())
    }
}
