// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::content_info::ContentInfo;
use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;
use crate::entity_kind::EntityKind;
use crate::named_entity_info::NamedEntityInfo;
use crate::property_info::PropertyInfo;
use crate::schema_info::SchemaInfo;

pub struct PropertyInfoImpl {
    // EntitytInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    undefined_properties: HashMap<String, Value>,

    // NamedEntityInfo
    name: Option<String>,

    // PropertyInfo
    schema: Option<Box<dyn SchemaInfo>>,
    writable: bool,
}

impl PropertyInfoImpl {
    /// Returns a new PropertyInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - The DTDL version used to define the property.
    /// * `id` - The identifier.
    /// * `child_of` - The identifier of the parent element in which this property is defined.
    /// * `defined_in` - The identifier of the partition in which this property is defined.
    /// * `schema` - The schema.
    /// * `writable` - Is the property writable?
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        name: Option<String>,
        schema: Option<Box<dyn SchemaInfo>>,
        writable: bool,
    ) -> Self {
        Self {
            dtdl_version,
            id,
            child_of,
            defined_in,
            undefined_properties: HashMap::<String, Value>::new(),
            name,
            schema,
            writable,
        }
    }
}

impl EntityInfo for PropertyInfoImpl {
    /// Returns the DTDL version.
    fn dtdl_version(&self) -> i32 {
        self.dtdl_version
    }

    /// Returns the identifier.
    fn id(&self) -> &Dtmi {
        &self.id
    }

    /// Returns the kind of entity/
    fn entity_kind(&self) -> EntityKind {
        EntityKind::Property
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

impl NamedEntityInfo for PropertyInfoImpl {
    /// Returns the name of the property.
    fn name(&self) -> &Option<String> {
        &self.name
    }
}

impl ContentInfo for PropertyInfoImpl {}

impl PropertyInfo for PropertyInfoImpl {
    /// Returns the schema.
    fn schema(&self) -> &Option<Box<dyn SchemaInfo>> {
        &self.schema
    }

    /// Returns whether the property is writable.
    fn writable(&self) -> bool {
        self.writable
    }
}

#[cfg(test)]
mod property_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::model_parser::DTDL_VERSION;
    use crate::primitive_schema_info_impl::PrimitiveSchemaInfoImpl;
    use serde_json;

    #[test]
    fn new_property_info_impl_test() -> Result<(), String> {
        let id_result: Option<Dtmi> = create_dtmi("dtmi:com:example:Thermostat;1.0");
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

        let schema_info_id: Option<Dtmi> = create_dtmi("dtmi:dtdl:class:String;2");
        assert!(schema_info_id.is_some());

        let boxed_schema_info = Box::new(PrimitiveSchemaInfoImpl::new(
            DTDL_VERSION,
            schema_info_id.unwrap(),
            None,
            None,
            EntityKind::String,
        ));

        let mut property_info = PropertyInfoImpl::new(
            DTDL_VERSION,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            Some(String::from("one")),
            Some(boxed_schema_info),
            true,
        );
        property_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        property_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(property_info.dtdl_version() == 2);
        assert!(property_info.id() == &id);
        assert!(property_info.child_of().is_some());
        assert!(property_info.child_of().clone().unwrap() == child_of);
        assert!(property_info.defined_in().is_some());
        assert!(property_info.defined_in().clone().unwrap() == defined_in);
        assert!(property_info.entity_kind() == EntityKind::Property);
        assert!(property_info.undefined_properties().len() == 2);
        assert!(
            property_info.undefined_properties().get("first").unwrap().clone()
                == first_propery_value
        );
        assert!(
            property_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );

        match property_info.name() {
            Some(name) => assert_eq!(name, "one"),
            None => return Err(String::from("name has not been set")),
        }

        match property_info.schema() {
            Some(schema) => assert_eq!(schema.entity_kind(), EntityKind::String),
            None => return Err(String::from("schema has not been set")),
        }

        assert!(property_info.writable());

        Ok(())
    }
}
