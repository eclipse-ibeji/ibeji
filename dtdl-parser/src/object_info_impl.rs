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
    fields: Vec<Box<dyn FieldInfo>>,
}

impl ObjectInfoImpl {
    /// Returns a new ObjectInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - Version of DTDL used to define the Entity.
    /// * `id` - Identifier for the Entity.
    /// * `child_of` - Identifier of the parent element in which this Entity is defined.
    /// * `defined_in` - Identifier of the partition in which this Entity is defined.
    /// * `fields` - The object's fields.
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        fields: Vec<Box<dyn FieldInfo>>
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

    /// Returns the identifier of the DTDL element that corresponds to this object.
    fn id(&self) -> &Dtmi {
        &self.id
    }

    /// Returns the kind of Entity, meaning the concrete DTDL type assigned to the corresponding element in the model.
    fn entity_kind(&self) -> EntityKind {
        EntityKind::Object
    }

    /// Returns the identifier of the parent DTDL element in which this element is defined.
    fn child_of(&self) -> &Option<Dtmi> {
        &self.child_of
    }

    /// Returns the identifier of the partition DTDL element in which this element is defined.
    fn defined_in(&self) -> &Option<Dtmi> {
        &self.defined_in
    }

    /// Returns any undefined properties of the DTDL element that corresponds to this object.
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

impl SchemaInfo for ObjectInfoImpl {
}

impl ComplexSchemaInfo for ObjectInfoImpl {
}

impl ObjectInfo for ObjectInfoImpl {
    // Returns the fields.
    fn fields(&self) -> &Vec<Box<dyn FieldInfo>> {
        &self.fields
    }
}

// #[cfg(test)]
/*
mod property_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use serde_json;

    #[test]
    fn new_property_info_impl_test() {
        let mut id_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:Thermostat;1.0", &mut id_result);
        assert!(id_result.is_some());
        let id = id_result.unwrap();

        let mut child_of_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:Cabin;1.0", &mut child_of_result);
        assert!(child_of_result.is_some());
        let child_of = child_of_result.unwrap();

        let mut defined_in_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:Something;1.0", &mut defined_in_result);
        assert!(defined_in_result.is_some());
        let defined_in = defined_in_result.unwrap();

        let first_propery_value: Value = serde_json::from_str("{\"first\": \"this\"}").unwrap();
        let second_propery_value: Value = serde_json::from_str("{\"second\": \"that\"}").unwrap();

        let mut property_info = PropertyInfoImpl::new(
            String::from("one"),
            2,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
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
            property_info.undefined_properties().get("first").unwrap().clone() == first_propery_value
        );
        assert!(
            property_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );

        assert!(property_info.name() == "one");        
    }
}
*/
