// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::component_info::ComponentInfo;
use crate::content_info::ContentInfo;
use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;
use crate::entity_kind::EntityKind;
use crate::interface_info::InterfaceInfo;
use crate::named_entity_info::NamedEntityInfo;

pub struct ComponentInfoImpl {
    // EntityInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    undefined_properties: HashMap<String, Value>,

    // NamedEntityInfo
    name: Option<String>,

    // ComponentInfo
    schema: Option<Box<dyn InterfaceInfo>>,
}

impl ComponentInfoImpl {
    /// Returns a new ComponentInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - The DTDL version used to define the component.
    /// * `id` - The identifier.
    /// * `child_of` - The identifier of the parent element in which this component is defined.
    /// * `defined_in` - The identifier of the partition in which this component is defined.
    /// * `name` - The name.
    /// * `schema` - The interface.
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        name: Option<String>,
        schema: Option<Box<dyn InterfaceInfo>>,
    ) -> Self {
        Self {
            dtdl_version,
            id,
            child_of,
            defined_in,
            undefined_properties: HashMap::<String, Value>::new(),
            name,
            schema,
        }
    }
}

impl EntityInfo for ComponentInfoImpl {
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
        EntityKind::Component
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

impl NamedEntityInfo for ComponentInfoImpl {
    /// Returns the name.
    fn name(&self) -> &Option<String> {
        &self.name
    }
}

impl ContentInfo for ComponentInfoImpl {}

impl ComponentInfo for ComponentInfoImpl {
    /// Returns the interface.
    fn schema(&self) -> &Option<Box<dyn InterfaceInfo>> {
        &self.schema
    }
}

#[cfg(test)]
mod component_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::interface_info_impl::InterfaceInfoImpl;
    use crate::model_parser::DTDL_VERSION;
    use serde_json;

    #[test]
    fn new_component_info_impl_test() -> Result<(), String> {
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

        let mut schema_info_id: Option<Dtmi> = None;
        create_dtmi("dtmi:dtdl:class:String;2", &mut schema_info_id);
        assert!(schema_info_id.is_some());

        let boxed_interface_info =
            Box::new(InterfaceInfoImpl::new(DTDL_VERSION, schema_info_id.unwrap(), None, None));

        let mut component_info = ComponentInfoImpl::new(
            DTDL_VERSION,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            Some(String::from("one")),
            Some(boxed_interface_info),
        );
        component_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        component_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(component_info.dtdl_version() == 2);
        assert!(component_info.id() == &id);
        assert!(component_info.child_of().is_some());
        assert!(component_info.child_of().clone().unwrap() == child_of);
        assert!(component_info.defined_in().is_some());
        assert!(component_info.defined_in().clone().unwrap() == defined_in);
        assert!(component_info.entity_kind() == EntityKind::Component);
        assert!(component_info.undefined_properties().len() == 2);
        assert!(
            component_info.undefined_properties().get("first").unwrap().clone()
                == first_propery_value
        );
        assert!(
            component_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );

        match component_info.name() {
            Some(name) => assert_eq!(name, "one"),
            None => return Err(String::from("name has not been set")),
        }

        match component_info.schema() {
            Some(schema) => assert_eq!(schema.entity_kind(), EntityKind::Interface),
            None => return Err(String::from("schema has not been set")),
        }

        Ok(())
    }
}
