// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::collections::HashMap;

use crate::dtmi::Dtmi;
use crate::entity_kind::EntityKind;

#[derive(Debug)]
pub struct EntityInfo {
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    entity_kind: EntityKind,
    undefined_properties: HashMap<String, Value>,
}

impl EntityInfo {
    /// Returns a new EntityInfo.
    ///
    /// # Arguments
    /// * `dtdl_version` - Version of DTDL used to define the Entity.
    /// * `id` - Identifier for the Entity.
    /// * `child_of` - Identifier of the parent element in which this Entity is defined.
    /// * `defined_in` - Identifier of the partition in which this Entity is defined.
    /// * `entity_kind` - The kind of Entity, which may be other than Entity if this constructor is called from a derived class.
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
            undefined_properties: HashMap::<String, Value>::new(),
        }
    }

    pub fn dtdl_version(&self) -> i32 {
        self.dtdl_version
    }

    /// Returns the identifier of the DTDL element that corresponds to this object.
    pub fn id(&self) -> &Dtmi {
        &self.id
    }

    /// Returns the kind of Entity, meaning the concrete DTDL type assigned to the corresponding element in the model.
    pub fn entity_kind(&self) -> &EntityKind {
        &self.entity_kind
    }

    // Returns the identifier of the parent DTDL element in which this element is defined.
    pub fn child_of(&self) -> &Option<Dtmi> {
        &self.child_of
    }

    // Returns the identifier of the partition DTDL element in which this element is defined.
    pub fn defined_in(&self) -> &Option<Dtmi> {
        &self.defined_in
    }

    // Returns any undefined properties of the DTDL element that corresponds to this object.
    pub fn undefined_properties(&self) -> &HashMap<String, Value> {
        &self.undefined_properties
    }

    // Add an undefined property.
    /// # Arguments
    /// * `key` - The property's name.
    /// * `value` - The property's value.
    pub fn add_undefined_property(&mut self, key: String, value: Value) {
        self.undefined_properties.insert(key, value);
    }
}

#[cfg(test)]
mod entity_info_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use serde_json;

    #[test]
    fn new_entity_info_test() {
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

        let mut entity_info = EntityInfo::new(
            2,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            EntityKind::Property,
        );
        entity_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        entity_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(entity_info.dtdl_version() == 2);
        assert!(entity_info.id() == &id);
        assert!(entity_info.child_of().is_some());
        assert!(entity_info.child_of().clone().unwrap() == child_of);
        assert!(entity_info.defined_in().is_some());
        assert!(entity_info.defined_in().clone().unwrap() == defined_in);
        assert!(*entity_info.entity_kind() == EntityKind::Property);
        assert!(entity_info.undefined_properties().len() == 2);
        assert!(
            entity_info.undefined_properties().get("first").unwrap().clone() == first_propery_value
        );
        assert!(
            entity_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );
    }
}
