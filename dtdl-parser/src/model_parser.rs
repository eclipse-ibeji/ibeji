// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use async_std::task;
use ibeji_common::find_full_path;
use json_ld::{context, Document, NoLoader, Node, Object};
use log::warn;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::command_info_impl::CommandInfoImpl;
use crate::command_payload_info::CommandPayloadInfo;
use crate::command_payload_info_impl::CommandPayloadInfoImpl;
use crate::component_info_impl::ComponentInfoImpl;
use crate::dtmi::{create_dtmi, Dtmi};
use crate::entity_info::EntityInfo;
use crate::entity_kind::{EntityKind, is_primitive_entity_kind};
use crate::field_info::FieldInfo;
use crate::field_info_impl::FieldInfoImpl;
use crate::interface_info::InterfaceInfo;
use crate::interface_info_impl::InterfaceInfoImpl;
use crate::json_ld::util::AsJson;
use crate::model_dict::ModelDict;
use crate::object_info_impl::ObjectInfoImpl;
use crate::primitive_schema_info_impl::PrimitiveSchemaInfoImpl;
use crate::property_info_impl::PropertyInfoImpl;
use crate::schema_info::SchemaInfo;
use crate::telemetry_info_impl::TelemetryInfoImpl;

/// The DTDL Version that the parser supports.
pub const DTDL_VERSION: i32 = 2;

/// Instances of the ModelParser class parse models written in the DTDL language.
/// This class can be used to determine whether one or more DTDL models are valid,
/// to identify specific modeling errors, and to enable inspection of model contents.
#[derive(Debug, Clone, Default)]
pub struct ModelParser {}

impl ModelParser {
    /// Returns a new ModelParser instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Parse a list of JSON texts and return the corresponding model.
    ///
    /// # Arguments
    /// * `json_texts` - A list of JSON texts.
    pub fn parse(&mut self, json_texts: &Vec<String>) -> Result<ModelDict, String> {
        let mut model: ModelDict = ModelDict::new();
        
        if let Err(message) = self.add_primitive_schemas_to_model(&mut model) {
            return Err(message)
        }

        // Add the entries to the model for the primitive entity kinds.
        for entity_kind in EntityKind::iter() {
            if is_primitive_entity_kind(entity_kind) {
                let mut schema_info_id: Option<Dtmi> = None;
                create_dtmi(&entity_kind.to_string(), &mut schema_info_id);
                if schema_info_id.is_none() {
                    return Err(format!("Cannot form a valid schema id for primitive entity kind '{}.", entity_kind.to_string()));            
                }

                let boxed_entity_info = Box::new(PrimitiveSchemaInfoImpl::new(DTDL_VERSION, schema_info_id.clone().unwrap(), None, None, entity_kind));
                model.insert(schema_info_id.clone().unwrap(), boxed_entity_info);
            }
        }

        for json_text in json_texts {
            let mut doc: Value = match serde_json::from_str(json_text) {
                Ok(json) => json,
                Err(error) => {
                    return Err(format!(
                        "Failed to parse one of the JSON texts due to: {:?}",
                        error
                    ))
                }
            };

            self.preprocess(&mut doc)?;

            let mut loader = NoLoader::<Value>::new();
            let dtdl_doc = match task::block_on(doc.expand::<context::Json<Value>, _>(&mut loader))
            {
                Ok(expanded_doc) => expanded_doc,
                Err(error) => {
                    return Err(format!(
                        "Failed to expand one of the JSON texts due to: {:?}",
                        error
                    ))
                }
            };

            for item in dtdl_doc.iter() {
                let object: &Object<serde_json::Value> = item;
                if let Object::Node(node) = object {
                    self.parse_node(node, &None, &mut model)?;
                }
            }
        }

        Ok(model)
    }

    /// Add the entries to the model for the primitive schemas.
    /// # Arguments
    /// * `model` - The model.
    fn add_primitive_schemas_to_model(&mut self, model: &mut ModelDict) -> Result<(), String>
    {
        for entity_kind in EntityKind::iter() {
            if is_primitive_entity_kind(entity_kind) {
                let mut schema_info_id: Option<Dtmi> = None;
                create_dtmi(&entity_kind.to_string(), &mut schema_info_id);
                if schema_info_id.is_none() {
                    return Err(format!("Cannot form a valid schema id for primitive schema '{}.", entity_kind.to_string()));            
                }

                let boxed_entity_info = Box::new(PrimitiveSchemaInfoImpl::new(DTDL_VERSION, schema_info_id.clone().unwrap(), None, None, entity_kind));
                model.insert(schema_info_id.clone().unwrap(), boxed_entity_info);
            }
        }

        Ok(())
    }

    /// Retrieve a JSON-LD context as a JSON object from the location specified in the filepath.
    ///
    /// # Arguments
    /// * `filepath` - The file path where the context is located.
    fn retrieve_context(&mut self, filepath: &Path) -> Result<Value, String> {
        let contents: String = match fs::read_to_string(filepath) {
            Ok(data) => data,
            Err(error) => {
                return Err(format!(
                    "Unable to read the context located at {} due to: {:?}",
                    filepath.display(),
                    error
                ))
            }
        };

        let doc: Value = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(error) => {
                return Err(format!(
                    "Unable to pasrse the context located at {} due to: {:?}",
                    filepath.display(),
                    error
                ))
            }
        };

        Ok(doc)
    }

    /// Replace a name reference for a context in a JSON object with its respective JSON value.
    ///
    /// # Arguments
    /// * `obj` - The JSON object represented as a map of names to JSON objects.
    /// `context_name` - The name of the context that we want to replace.
    /// 'context_value` - The JSON object that we will replace it with.
    #[allow(clippy::needless_range_loop)]
    fn replace_context_inline_in_object(
        &mut self,
        obj: &mut Map<String, Value>,
        context_name: &str,
        context_value: &Value,
    ) -> Result<(), String> {
        let existing_context_value_option = obj.get_mut("@context");
        if let Some(existing_context_value) = existing_context_value_option {
            if let Value::String(s) = existing_context_value {
                if s == context_name {
                    obj.remove("@context");
                    obj.insert(String::from("@context"), context_value.clone());
                }
            } else if let Value::Array(a) = existing_context_value {
                for i in 0..a.len() {
                    if let Value::String(_s) = &a[i] {
                        if a[i] == context_name {
                            a[i] = context_value.clone();
                            break;
                        }
                    }
                }
            } else if let Value::Object(_o) = existing_context_value {
                // ignore - this one does not have an IRI associated with it.
            } else {
                return Err(format!("Unexpected context value '{:?}'", existing_context_value));
            }
        }
        Ok(())
    }

    /// Replace a name reference for a context in a JSON-LD document with its respective JSON value.
    ///
    /// # Arguments
    /// * `doc` - The JSON document.
    /// `context_name` - The name of the context that we want to replace.
    /// 'context_value` - The JSON object that we will replace it with.
    fn replace_context_inline_in_doc(
        &mut self,
        doc: &mut Value,
        context_name: &str,
        context_value: &Value,
    ) -> Result<(), String> {
        match doc {
            Value::Array(array) => {
                for v in array.iter_mut() {
                    self.replace_context_inline_in_doc(v, context_name, context_value)?;
                }
            }
            Value::Object(object) => {
                self.replace_context_inline_in_object(object, context_name, context_value)?;
            }
            _ => warn!("An unexpected json value was encountered"),
        }
        Ok(())
    }

    /// Preprocess a JSON-LD document, so that supported dtmi contexts will have their names replaced by their respective JSON.
    ///
    /// # Arguments
    /// `doc` - The JSON-LD document to preprocess.
    ///
    /// # Examples of supported context formats:
    ///
    /// "@context": "https://json-ld.org/contexts/person.json"
    ///
    /// "@context": [
    /// "https://json-ld.org/contexts/person.json",
    /// "https://json-ld.org/contexts/place.json",
    /// {"title": "http://purl.org/dc/terms/title"}
    /// ]
    fn preprocess(&mut self, doc: &mut Value) -> Result<(), String> {
        let dtdl_2_context_path_string = find_full_path("v2/context/DTDL.v2.context.json")?;
        let dtdl_2_context_path_string_unwrapped = dtdl_2_context_path_string;
        let dtdl_2_context_path = Path::new(&dtdl_2_context_path_string_unwrapped);
        let dtdl_2_context_value = self.retrieve_context(dtdl_2_context_path)?;
        self.replace_context_inline_in_doc(doc, "dtmi:dtdl:context;2", &dtdl_2_context_value)?;

        let sdv_2_context_path_string = find_full_path("v2/context/SDV.v2.context.json")?;
        let sdv_2_context_path_string_unwrapped = sdv_2_context_path_string;
        let sdv_2_context_path = Path::new(&sdv_2_context_path_string_unwrapped);
        let sdv_2_context_value = self.retrieve_context(sdv_2_context_path)?;
        self.replace_context_inline_in_doc(doc, "dtmi:sdv:context;2", &sdv_2_context_value)?;

        Ok(())
    }

    /// Get a property value from a node by name.
    ///
    /// #Arguments
    /// `node` - The node that contains the property.
    /// `property_name` - The name of the property.
    fn get_property_value(&self, node: &Node<Value>, property_name: &str) -> Result<Option<String>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == property_name {
                if the_objects.len() == 1 {
                    match the_objects[0].as_str() {
                        Some(v) => return Ok(Some(String::from(v))),
                        None => return Err(format!("get_property_value was unable to convert the value to a str"))
                    }
                } else {
                    return Err(format!("get_property_value does not contain the expected number of objects"));
                }
            }
        }

        Ok(None)
    }

    fn get_primary_schema(&self, node: &Node<Value>, parent_id: &Option<Dtmi>) -> Result<Box<dyn SchemaInfo>, String> {
        let entity_kind;

        let string_option: Option<&str> = node.as_str();
        if string_option.is_some() {
            entity_kind = match EntityKind::from_str(string_option.unwrap()) {
                Ok(v) => v,
                Err(e) => return Err(format!("{:?}", e))
            };

            if !is_primitive_entity_kind(entity_kind) {
                return Err(format!("unable to get the schema, as we found an unexpected primitive type."));
            }
        } else {
            return Err(format!("get_schema encountered an unknown entity kind value"));                            
        }

        let id: Option<Dtmi> = self.generate_id(parent_id, "test");
        if id.is_none() {
            return Err(String::from("We were not able to generate an id for the schema."));
        }

        Ok(Box::new(PrimitiveSchemaInfoImpl::new(
            DTDL_VERSION,
            id.unwrap(),
            parent_id.clone(),
            None,
            entity_kind)))
    }

    fn get_object_schema(&self, node: &Node<Value>, parent_id: &Option<Dtmi>) -> Result<Box<dyn SchemaInfo>, String> {
        let mut fields: Vec<Box<dyn FieldInfo>> = Vec::new();

        for (the_property, the_objects) in node.properties() {
            if the_property == "dtmi:dtdl:property:fields;2" {
                let mut i = 0;
                while i < the_objects.len() {
                    if let Object::Node(node) = &*the_objects[i] {
                        let mut name_option: Option<String> = None;
                        let mut _display_name_option: Option<String> = None;
                        let mut schema: Option<Box<dyn SchemaInfo>> = None;
                        for (the_property, the_objects) in node.properties() {
                            if the_property == "dtmi:dtdl:property:displayName;2" && the_objects.len() == 1 {
                                if let Object::Value(value) = &*the_objects[0] {
                                    match value.as_str() {
                                        Some(value) => _display_name_option = Some(String::from(value)),
                                        None => _display_name_option = None,
                                    }                                    
                                }
                            } else if the_property == "dtmi:dtdl:property:schema;2" && the_objects.len() == 1 {
                                if let Object::Node(node) = &*the_objects[0] {                                   
                                    schema = Some(self.get_schema(node, parent_id)?);
                                }
                            } else if the_property == "dtmi:dtdl:property:name;2" && the_objects.len() == 1 {
                                if let Object::Value(value) = &*the_objects[0] {
                                    match value.as_str() {
                                        Some(value) => name_option = Some(String::from(value)),
                                        None => name_option = None,
                                    }                                                                       
                                }
                            }
                        }
                        if name_option.is_some() {
                            let id: Option<Dtmi> = self.generate_id(parent_id, &name_option.clone().unwrap());
                            if id.is_none() {
                                return Err(String::from("We were not able to generate an id for the schema."));
                            }

                            fields.push(Box::new(FieldInfoImpl::new(
                                name_option.clone().unwrap(),
                                DTDL_VERSION,
                                id.unwrap(),
                                parent_id.clone(),
                                None,
                                schema
                            )));
                        }
                    }
                    i+=1;
                }
            }
        }

        let id: Option<Dtmi> = self.generate_id(parent_id, "test");
        if id.is_none() {
            return Err(String::from("We were not able to generate an id for the schema."));
        }

        Ok(Box::new(ObjectInfoImpl::new(
            DTDL_VERSION,
            id.unwrap(),
            parent_id.clone(),
            None,
            fields)))
    }

    fn get_complex_schema(&self, node: &Node<Value>, parent_id: &Option<Dtmi>) -> Result<Box<dyn SchemaInfo>, String> {
        let mut entity_kind_option: Option<EntityKind> = None;
        for node_type in node.types() {
            let entity_kind_result = EntityKind::from_str(node_type.as_str());
            if let Ok(_entity_kind) = entity_kind_result {
                entity_kind_option = Some(entity_kind_result.unwrap());
                break;
            }
        }

        if entity_kind_option.is_none() {
            return Err(format!("Complex schema has no associated type.  It must have one."));
        }

        let entity_kind = entity_kind_option.unwrap();

        if entity_kind == EntityKind::Object {
            return self.get_object_schema(node, parent_id);
        } else {
            return Err(format!("Unsupported complex object: {:?}.", entity_kind));
        }
    }

    fn get_schema(&self, node: &Node<Value>, parent_id: &Option<Dtmi>) -> Result<Box<dyn SchemaInfo>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == "dtmi:dtdl:property:schema;2" {
                if the_objects.len() == 1 {
                    if let Object::Node(node) = &*the_objects[0] {
                        if node.properties().len() == 0 {
                            return self.get_primary_schema(node, parent_id);                       
                        } else {
                            return self.get_complex_schema(node, parent_id);

                        }
                    } else {
                        return Err(format!("The schema property's associated object should be a node.  It is not."));
                    }
                } else {
                    return Err(format!("The schema property should only have 1 assoicated object.  It has {}.", the_objects.len()));
                }
            }
        }

        Err(format!("A schema property was not found."))
    }

    fn get_payload(&self, node: &Node<Value>, property_name: &str, parent_id: &Option<Dtmi>) -> Result<Option<Box<dyn CommandPayloadInfo>>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == property_name {
                if let Object::Node(node) = &*the_objects[0] {
                    // name - required
                    let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
                    if name.is_none() {
                        return Err(String::from("Command does not have a name property."));
                    }

                    let mut id: Option<Dtmi> = None;
                    if node.id().is_some() {
                        create_dtmi(node.id().unwrap().as_str(), &mut id);
                    }
                    if id.is_none() {
                        id = self.generate_id(parent_id, &name.clone().unwrap());
                        if id.is_none() {
                            return Err(String::from("We were not able to generate an id for the payload."));
                        }
                    }

                    // displayName - required
                    let _display_name = self.get_property_value(node, "dtmi:dtdl:property:displayName;2")?;
                    
                    // description - required
                    let _description = self.get_property_value(node, "dtmi:dtdl:property:description;2")?;

                    // schema - required
                    let boxed_schema_info: Box<dyn SchemaInfo> = self.get_schema(node, &id)?;
                    
                    return Ok(Some(Box::new(CommandPayloadInfoImpl::new(
                        name.unwrap(),
                        DTDL_VERSION,
                        id.unwrap(),
                        parent_id.clone(),
                        None,
                        Some(boxed_schema_info)))));

                } else {
                    return Err(format!("get_payload encountered an unknown object"));
                }
            }
        }

        Ok(None)
    }

    /// Gather the undefined propeties from a node.
    ///
    /// Arguments
    /// `node` - The node to gather the undefined properties from.
    fn gather_undefined_properties(
        node: &Node<Value>,
        undefined_properties: &mut HashMap<String, Value>,
    ) {
        for (the_property, the_objects) in node.properties() {
            if the_objects.len() == 1 {
                if let Object::Value(value) = &*the_objects[0] {
                    let j = value.clone().as_json();
                    undefined_properties.insert(the_property.to_string(), j);
                } else if let Object::Node(n) = &*the_objects[0] {
                    Self::gather_undefined_properties(n, undefined_properties);
                } else if let Object::List(_list) = &*the_objects[0] {
                    warn!("gather_undefiued_properties encountered a list");
                } else {
                    warn!("Warning: gather_undefiued_properties encountered an unknonw object");
                }
            }
        }
    }

    /// Genrate an id from the associated parent id and the associated property name.
    ///
    /// # Arguments
    /// `parent_id` - The associated parent id.
    /// `name` - The associated property name.
    fn generate_id(&self, parent_id: &Option<Dtmi>, name: &str) -> Option<Dtmi> {
        let generated_id_value = format!("{}:{}", parent_id.clone().unwrap().versionless(), name);
        let mut generated_id: Option<Dtmi> = None;
        create_dtmi(&generated_id_value, &mut generated_id);
        generated_id
    }

    /// Retrieve a schema info from a model.
    ///
    /// # Arguments
    /// `schema` - The id (as a string) for the schema info.
    /// `model` - The model to search.
    #[allow(dead_code)]
    fn retrieve_schema_info_from_model(
        &mut self,
        schema: &str,
        model: &mut ModelDict,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        let mut primitive_schema_info_id: Option<Dtmi> = None;
        create_dtmi(schema, &mut primitive_schema_info_id);
        if primitive_schema_info_id.is_none() {
            return Err(String::from("Primitive schema cannot form a valid schema id."));            
        }
        let primitive_schema_info_model_entry = model.get(&primitive_schema_info_id.clone().unwrap());
        if primitive_schema_info_model_entry.is_none() {
            return Err(format!("We were not able to find the primitive schema entry for id '{}'.", primitive_schema_info_id.clone().unwrap()));
        }
        let boxed_primitive_schema_info_ref = primitive_schema_info_model_entry.unwrap().as_any()
            .downcast_ref::<PrimitiveSchemaInfoImpl>()
            .expect("Was not a primitive schema info");
        let boxed_schema_info: Box<dyn SchemaInfo> = Box::new((*boxed_primitive_schema_info_ref).clone());
        
        Ok(boxed_schema_info)
    }

    /// Retrieve an interface info from a model.
    ///
    /// # Arguments
    /// `schema` - The id (as a string) for the interface info.
    /// `model` - The model to search.
    fn retrieve_interface_info_from_model(
        &mut self,
        schema: &str,
        model: &mut ModelDict,
    ) -> Result<Box<dyn InterfaceInfo>, String> {
        let mut interface_info_id: Option<Dtmi> = None;
        create_dtmi(schema, &mut interface_info_id);
        if interface_info_id.is_none() {
            return Err(String::from("Schema cannot form a valid schema id."));            
        }
        let interface_info_model_entry = model.get(&interface_info_id.clone().unwrap());
        if interface_info_model_entry.is_none() {
            return Err(format!("We were not able to find the interface entry for id '{}'.", interface_info_id.clone().unwrap()));
        }
        let _boxed_interface_info_ref = interface_info_model_entry.unwrap().as_any()
            .downcast_ref::<InterfaceInfoImpl>()
            .expect("Was not an ineterface info");
        let boxed_interface_info: Box<dyn InterfaceInfo> = Box::new((*_boxed_interface_info_ref).clone());
        
        Ok(boxed_interface_info)
    }    

    /// Parse a node.
    ///
    /// # Arguments
    /// `node` - The node to parse.
    /// `model` - The model to add the content to.
    fn parse_node(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        let mut entity_kind_option: Option<EntityKind> = None;
        for node_type in node.types() {
            let entity_kind_result = EntityKind::from_str(node_type.as_str());
            if let Ok(_entity_kind) = entity_kind_result {
                entity_kind_option = Some(entity_kind_result.unwrap());
                break;
            }
        }

        if entity_kind_option.is_none() {
            return Err(String::from("Warning: No entity kind found amongst the node's types"));
        }

        match entity_kind_option.unwrap() {
            EntityKind::Interface => self.parse_interface(node, parent_id, model)?,
            EntityKind::Telemetry => self.parse_telemetry(node, parent_id, model)?,
            EntityKind::Property => self.parse_property(node, parent_id, model)?,
            EntityKind::Command => self.parse_command(node, parent_id, model)?,
            EntityKind::Relationship => self.parse_relationship(node, parent_id, model)?,
            EntityKind::Component => self.parse_component(node, parent_id, model)?,
            _ => return Err(String::from("Warning: Unexepcted entity kind found "))
        }

        Ok(())
    }

    /// Parse an interface.
    ///
    /// # Arguments
    /// `node` - The node that represents an interface.
    /// `parent_id` - The interface's parent id.
    /// `model` - The model to add the content to.
    fn parse_interface(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // @id - required
        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            return Err(format!(
                "Interface does not have a valid id '{}'",
                node.id().unwrap().as_str()
            ));
        }

        // contents - optional
        for (the_property, the_objects) in node.properties() {
            if the_property != "dtmi:dtdl:property:contents;2" {
                continue;
            }
            for the_object in the_objects {
                let object: &Object<serde_json::Value> = the_object;
                if let Object::Node(node) = object {
                    self.parse_node(node, &id, model)?;
                }
            }
        }

        // Add the interface to the object model.
        let entity_info = Box::new(InterfaceInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
        ));
        model.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }

    /// Parse a telemetry.
    ///
    /// # Arguments
    /// `node` - The node that represents a telemetry.
    /// `parent_id` - The telemetry's parent id.
    /// `model` - The model to add the content to.
    fn parse_telemetry(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // name - required
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
        if name.is_none() {
            return Err(String::from("Telemetry does not have a name property."));
        }

        // schema - required
        let boxed_schema_info: Box<dyn SchemaInfo> = self.get_schema(node, parent_id)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the telemetry."));
            }
        }

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);   

        let mut rc_entity_info: Box<dyn EntityInfo> = Box::new(TelemetryInfoImpl::new(
            name.unwrap(),
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            Some(boxed_schema_info)
        ));

        for (key, value) in undefined_property_values {
            rc_entity_info.add_undefined_property(key, value);
        }

        model.insert(id.clone().unwrap(), rc_entity_info);

        Ok(())
    }

    /// Parse a property.
    ///
    /// # Arguments
    /// `node` - The node that represents a property.
    /// `parent_id` - The property's parent id.
    /// `model` - The model to add the content to.
    fn parse_property(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // name - required
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
        if name.is_none() {
            return Err(String::from("Property does not have a name property."));
        }

        // schema - required
        let boxed_schema_info: Box<dyn SchemaInfo> = self.get_schema(node, parent_id)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the property."));
            }
        }

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);

        let mut entity_info = Box::new(PropertyInfoImpl::new(
            name.unwrap(),
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            Some(boxed_schema_info),
            false,
        ));

        for (key, value) in undefined_property_values {
            entity_info.add_undefined_property(key, value);
        }

        model.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }

    /// Parse a command.
    ///
    /// # Arguments
    /// `node` - The node that represents a command.
    /// `parent_id` - The command's parent id.
    /// `model` - The model to add the content to.
    fn parse_command(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // name - required
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
        if name.is_none() {
            return Err(String::from("Command does not have a name property."));
        }

        println!("Command: {}", name.clone().unwrap());

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the command."));
            }
        }

        let request_payload: Option<Box<dyn CommandPayloadInfo>> = self.get_payload(node, "dtmi:dtdl:property:request;2", &id)?;
        let response_payload: Option<Box<dyn CommandPayloadInfo>> = self.get_payload(node, "dtmi:dtdl:property:response;2", &id)?;

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);        

        let mut entity_info = Box::new(CommandInfoImpl::new(
            name.unwrap(),
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            request_payload,
            response_payload,
        ));

        for (key, value) in undefined_property_values {
            entity_info.add_undefined_property(key, value);
        }
            
        model.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }

    /// Parse a relationship.
    ///
    /// # Arguments
    /// `node` - The node that represents a relationship.
    /// `parent_id` - The relationship's parent id.
    /// `model` - The model to add the content to.
    fn parse_relationship(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // name - required
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
        if name.is_none() {
            return Err(String::from("Relationship does not have a name property."));
        }

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from(
                    "We were not able to generate an id for the relationship.",
                ));
            }
        }

        let entity_info = Box::new(PropertyInfoImpl::new(
            name.unwrap(),
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            None,
            false,
        ));
        model.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }

    // Parse a component.
    ///
    /// # Arguments
    /// `node` - The node that represents a component.
    /// `parent_id` - The component's parent id.
    /// `model` - The model to add the content to.
    fn parse_component(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model: &mut ModelDict,
    ) -> Result<(), String> {
        // name - required
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;
        if name.is_none() {
            return Err(String::from("Component does not have a name property"));
        }

        // schema - required (note: here the schema property represents an interface)
        let schema = self.get_property_value(node, "dtmi:dtdl:property:schema;2")?;
        if schema.is_none() {
            return Err(String::from("Component does not have a schema property."));
        }
        let boxed_interface_info: Box<dyn InterfaceInfo> = self.retrieve_interface_info_from_model(&schema.unwrap(), model)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            create_dtmi(node.id().unwrap().as_str(), &mut id);
        }
        if id.is_none() {
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the component."));
            }
        }

        let entity_info = Box::new(ComponentInfoImpl::new(
            name.unwrap(),            
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            Some(boxed_interface_info),
        ));
        model.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }
}

#[cfg(test)]
mod model_parser_tests {
    use super::*;
    use ibeji_common_test::set_dtdl_path;
    use std::fs;
    use std::path::Path;
    use std::vec::Vec;

    fn retrieve_dtdl(file_path: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        let read_result = fs::read_to_string(path);
        match read_result {
            Ok(contents) => Ok(contents),
            Err(error) => Err(format!("Unable to retrieve the DTDL due to: {:?}", error)),
        }
    }

    #[test]
    fn validation_test() {
        set_dtdl_path();

        let mut json_texts = Vec::<String>::new();

        let device_information_full_path_result = find_full_path("dtmi/azure/devicemanagement/deviceinformation-1.json");
        assert!(device_information_full_path_result.is_ok());
        let device_information_contents_result = retrieve_dtdl(&device_information_full_path_result.unwrap());
        assert!(device_information_contents_result.is_ok());
        json_texts.push(device_information_contents_result.unwrap());           

        let thermostat_full_path_result = find_full_path("v2/samples/Thermostat.json");
        assert!(thermostat_full_path_result.is_ok());
        let thermostat_contents_result = retrieve_dtdl(&thermostat_full_path_result.unwrap());
        assert!(thermostat_contents_result.is_ok());
        json_texts.push(thermostat_contents_result.unwrap());        

        let temp_controller_full_path_result =
            find_full_path("v2/samples/TemperatureController.json");
        assert!(temp_controller_full_path_result.is_ok());
        let temp_controller_contents_result =
            retrieve_dtdl(&temp_controller_full_path_result.unwrap());
        assert!(temp_controller_contents_result.is_ok());
        json_texts.push(temp_controller_contents_result.unwrap());

        let mut parser = ModelParser::new();
        let model_result = parser.parse(&json_texts);
        assert!(model_result.is_ok(), "parse failed due to: {}", model_result.err().unwrap());
        let model = model_result.unwrap();
        assert!(model.len() == 31, "expected length was 31, actual length is {}", model.len());
    }

    #[test]
    fn demo_validation_test() {
        set_dtdl_path();

        let mut json_texts = Vec::<String>::new();

        let demo_path_result =
            find_full_path("samples/demo_resources.json");
        assert!(demo_path_result.is_ok());
        let demo_contents_result = retrieve_dtdl(&demo_path_result.unwrap());
        assert!(demo_contents_result.is_ok());
        json_texts.push(demo_contents_result.unwrap());

        let mut parser = ModelParser::new();
        let model_result = parser.parse(&json_texts);
        assert!(model_result.is_ok(), "parse failed due to: {}", model_result.err().unwrap());
        let model = model_result.unwrap();
        assert!(model.len() == 14, "expected length was 14, actual length is {}", model.len());        

        let mut ambient_air_temperature_id: Option<Dtmi> = None;
        create_dtmi(
            "dtmi:org:eclipse:sdv:property:cabin:AmbientAirTemperature;1",
            &mut ambient_air_temperature_id,
        );
        assert!(ambient_air_temperature_id.is_some());
        let ambient_air_temperature_entity_result = model.get(&ambient_air_temperature_id.unwrap());
        assert!(ambient_air_temperature_entity_result.is_some());
        let ambient_air_temperature_uri_property_result = ambient_air_temperature_entity_result.unwrap().undefined_properties().get("dtmi:sdv:property:uri;1");
        assert!(ambient_air_temperature_uri_property_result.is_some());
        let ambient_air_temperature_uri_property_value_result = ambient_air_temperature_uri_property_result.unwrap().get("@value");
        assert!(ambient_air_temperature_uri_property_value_result.is_some());
        assert!(ambient_air_temperature_uri_property_value_result.unwrap() == "http://[::1]:40010"); // Devskim: ignore DS137138

        let mut send_notification_id: Option<Dtmi> = None;
        create_dtmi(
            "dtmi:org:eclipse:sdv:command:HVAC:send_notification;1",
            &mut send_notification_id,
        );
        assert!(send_notification_id.is_some());
        let send_notification_entity_result = model.get(&send_notification_id.unwrap());
        assert!(send_notification_entity_result.is_some());
        let send_notification_uri_property_result = send_notification_entity_result.unwrap().undefined_properties().get("dtmi:sdv:property:uri;1");
        assert!(send_notification_uri_property_result.is_some());
        let send_notification_uri_property_value_result = send_notification_uri_property_result.unwrap().get("@value");
        assert!(send_notification_uri_property_value_result.is_some());
        assert!(send_notification_uri_property_value_result.unwrap() == "http://[::1]:40010"); // Devskim: ignore DS137138        
    }
}
