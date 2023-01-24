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
use crate::schema_info::SchemaInfo;
use crate::telemetry_info::TelemetryInfo;

pub struct TelemetryInfoImpl {
    // EntityInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    undefined_properties: HashMap<String, Value>,

    // NamedEntityInfo
    name: Option<String>,    

    // TelemetryInfo
    schema: Option<Box<dyn SchemaInfo>>,
}

impl TelemetryInfoImpl {
    /// Returns a new TelemetryInfoImpl.
    ///
    /// # Arguments
    /// * `dtdl_version` - The DTDL version used to define the telemetry.
    /// * `id` - The identifier.
    /// * `child_of` - The identifier of the parent element in which this telemetry is defined.
    /// * `defined_in` - The identifier of the partition in which this telemetry is defined.
    /// * `name` - The name.
    /// * `schema` - The schema.
    pub fn new(
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        name: Option<String>,        
        schema: Option<Box<dyn SchemaInfo>>
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

impl EntityInfo for TelemetryInfoImpl {
    // Returns the DTDL version.
    fn dtdl_version(&self) -> i32 {
        self.dtdl_version
    }

    /// Returns the identifier.
    fn id(&self) -> &Dtmi {
        &self.id
    }

    /// Returns the kind of entity.
    fn entity_kind(&self) -> EntityKind {
        EntityKind::Telemetry
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

impl NamedEntityInfo for TelemetryInfoImpl {  
    /// Returns the name.
    fn name(&self) -> &Option<String> {
        &self.name
    }  
}

impl ContentInfo for TelemetryInfoImpl {    
}

impl TelemetryInfo for TelemetryInfoImpl {
    /// Returns the schema.
    fn schema(&self) -> &Option<Box<dyn SchemaInfo>> {
        &self.schema
    }    
}

#[cfg(test)]
mod telemetry_info_impl_tests {
    use super::*;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::model_parser::DTDL_VERSION;
    use crate::primitive_schema_info_impl::PrimitiveSchemaInfoImpl;
    use serde_json;

    #[test]
    fn new_telemetry_info_impl_test() {
        let mut id_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:Thermostat;1.0", &mut id_result);
        assert!(id_result.is_some());
        let id = id_result.unwrap();

        let mut child_of_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:Cabin;1.0", &mut child_of_result);
        assert!(child_of_result.is_some());
        let child_of = child_of_result.unwrap();

        let mut defined_in_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example;1.0", &mut defined_in_result);
        assert!(defined_in_result.is_some());
        let defined_in = defined_in_result.unwrap();

        let first_propery_value: Value = serde_json::from_str("{\"first\": \"this\"}").unwrap();
        let second_propery_value: Value = serde_json::from_str("{\"second\": \"that\"}").unwrap();

        let mut schema_info_id: Option<Dtmi> = None;
        create_dtmi("dtmi:dtdl:class:String;2", &mut schema_info_id);
        assert!(schema_info_id.is_some());

        let boxed_schema_info = Box::new(PrimitiveSchemaInfoImpl::new(DTDL_VERSION, schema_info_id.unwrap(), None, None, EntityKind::String));        

        let mut telemetry_info = TelemetryInfoImpl::new(
            DTDL_VERSION,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            Some(String::from("one")),            
            Some(boxed_schema_info),
        );
        telemetry_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        telemetry_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(telemetry_info.dtdl_version() == 2);
        assert!(telemetry_info.id() == &id);
        assert!(telemetry_info.child_of().is_some());
        assert!(telemetry_info.child_of().clone().unwrap() == child_of);
        assert!(telemetry_info.defined_in().is_some());
        assert!(telemetry_info.defined_in().clone().unwrap() == defined_in);
        assert!(telemetry_info.entity_kind() == EntityKind::Telemetry);
        assert!(telemetry_info.undefined_properties().len() == 2);
        assert!(
            telemetry_info.undefined_properties().get("first").unwrap().clone() == first_propery_value
        );
        assert!(
            telemetry_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );
        
        match telemetry_info.name() {
            Some(name) => assert_eq!(name, "one"),
            None => assert!(false, "name has not been set")
        }

        match telemetry_info.schema() {
            Some(schema) => assert_eq!(schema.entity_kind(), EntityKind::String),
            None => assert!(false, "schema has not been set")
        }         
    }
}
