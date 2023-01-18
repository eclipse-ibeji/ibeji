// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::command_info::CommandInfo;
use crate::command_payload_info::CommandPayloadInfo;
use crate::content_info::ContentInfo;
use crate::dtmi::Dtmi;
use crate::entity_info::EntityInfo;
use crate::entity_kind::EntityKind;
use crate::named_entity_info::NamedEntityInfo;

pub struct CommandInfoImpl {
    // EntityInfo
    dtdl_version: i32,
    id: Dtmi,
    child_of: Option<Dtmi>,
    defined_in: Option<Dtmi>,
    undefined_properties: HashMap<String, Value>,

    // NamedEntityInfo
    name: String,

    // CommandInfo
    request: Option<Box<dyn CommandPayloadInfo>>,
    response: Option<Box<dyn CommandPayloadInfo>>,    
}

impl CommandInfoImpl {
    /// Returns a new CommandInfoImpl.
    ///
    /// # Arguments
    /// * `name` - Name of the command.
    /// * `dtdl_version` - Version of DTDL used to define the Entity.
    /// * `id` - Identifier for the Entity.
    /// * `child_of` - Identifier of the parent element in which this Entity is defined.
    /// * `defined_in` - Identifier of the partition in which this Entity is defined.
    /// * `request` - The request.
    pub fn new(
        name: String,
        dtdl_version: i32,
        id: Dtmi,
        child_of: Option<Dtmi>,
        defined_in: Option<Dtmi>,
        request: Option<Box<dyn CommandPayloadInfo>>,
        response: Option<Box<dyn CommandPayloadInfo>>        
    ) -> Self {
        Self {
            name,
            dtdl_version,
            id,
            child_of,
            defined_in,
            undefined_properties: HashMap::<String, Value>::new(),
            request,
            response
        }
    }
}

impl EntityInfo for CommandInfoImpl {
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
        EntityKind::Command
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

impl NamedEntityInfo for CommandInfoImpl {
    /// Returns the name of the telemetry.
    fn name(&self) -> &str {
        &self.name
    }
}

impl ContentInfo for CommandInfoImpl {

}

impl CommandInfo for CommandInfoImpl {
    /// Returns the request.
    fn request(&self) -> &Option<Box<dyn CommandPayloadInfo>> {
        &self.request
    }

    /// Returns the response.
    fn response(&self) -> &Option<Box<dyn CommandPayloadInfo>> {
        &self.response
    }
}

#[cfg(test)]
mod command_info_impl_tests {
    use super::*;
    use crate::command_payload_info_impl::CommandPayloadInfoImpl;
    use crate::dtmi::{create_dtmi, Dtmi};
    use crate::model_parser::DTDL_VERSION;
    use crate::primitive_schema_info_impl::PrimitiveSchemaInfoImpl;
    use serde_json;

    #[test]
    fn new_command_info_impl_test() {
        let mut id_result: Option<Dtmi> = None;
        create_dtmi("dtmi:com.example:command:HVAC:send_notification;1", &mut id_result);
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
        let string_schema_info = Box::new(PrimitiveSchemaInfoImpl::new(DTDL_VERSION, schema_info_id.unwrap(), None, None, EntityKind::String));   

        let mut request_id: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:send_notification:request:1", &mut request_id);
        assert!(request_id.is_some());
        let request = Box::new(CommandPayloadInfoImpl::new(String::from("send_notification"), DTDL_VERSION, request_id.unwrap(), None, None, Some(string_schema_info.clone())));   

        let mut response_id: Option<Dtmi> = None;
        create_dtmi("dtmi:com:example:send_notification:response:1", &mut response_id);
        assert!(response_id.is_some());
        let response = Box::new(CommandPayloadInfoImpl::new(String::from("send_notification"), DTDL_VERSION, response_id.unwrap(), None, None, Some(string_schema_info)));

        let mut command_info = CommandInfoImpl::new(
            String::from("one"),
            2,
            id.clone(),
            Some(child_of.clone()),
            Some(defined_in.clone()),
            Some(request),
            Some(response)
        );
        command_info.add_undefined_property(String::from("first"), first_propery_value.clone());
        command_info.add_undefined_property(String::from("second"), second_propery_value.clone());

        assert!(command_info.dtdl_version() == 2);
        assert!(command_info.id() == &id);
        assert!(command_info.child_of().is_some());
        assert!(command_info.child_of().clone().unwrap() == child_of);
        assert!(command_info.defined_in().is_some());
        assert!(command_info.defined_in().clone().unwrap() == defined_in);
        assert!(command_info.entity_kind() == EntityKind::Command);
        assert!(command_info.undefined_properties().len() == 2);
        assert!(
            command_info.undefined_properties().get("first").unwrap().clone() == first_propery_value
        );
        assert!(
            command_info.undefined_properties().get("second").unwrap().clone()
                == second_propery_value
        );

        assert!(command_info.name() == "one");
    }
}
