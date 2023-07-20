// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use futures::executor::block_on;
use json_ld::{context, Document, NoLoader, Node, Object};
use log::warn;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use strum::IntoEnumIterator;

use crate::command_info_impl::CommandInfoImpl;
use crate::command_payload_info::CommandPayloadInfo;
use crate::command_payload_info_impl::CommandPayloadInfoImpl;
use crate::component_info_impl::ComponentInfoImpl;
use crate::dtmi::{create_dtmi, Dtmi};
use crate::entity_kind::EntityKind;
use crate::field_info::FieldInfo;
use crate::field_info_impl::FieldInfoImpl;
use crate::interface_info::InterfaceInfo;
use crate::interface_info_impl::InterfaceInfoImpl;
use crate::json_ld::util::AsJson;
use crate::model_dict::ModelDict;
use crate::object_info_impl::ObjectInfoImpl;
use crate::primitive_schema_info_impl::PrimitiveSchemaInfoImpl;
use crate::primitive_schema_kinds::is_primitive_schema_kind;
use crate::property_info_impl::PropertyInfoImpl;
use crate::relationship_info_impl::RelationshipInfoImpl;
use crate::schema_info::SchemaInfo;
use crate::telemetry_info_impl::TelemetryInfoImpl;

/// The DTDL Version that the parser supports.
pub const DTDL_VERSION: i32 = 2;

/// Instances of the ModelParser class parse models written in the DTDL language.
/// This class can be used to determine: whether one or more DTDL models are valid,
/// to identify specific modeling errors, and to enable inspection of model contents.
pub struct ModelParser {}

impl Default for ModelParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelParser {
    /// The DTDL-path environment variable name.
    pub const DTDL_PATH: &str = "DTDL_PATH";

    /// Returns a new ModelParser instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Parse a list of JSON texts and return the resulting model dictionary.
    ///
    /// # Arguments
    /// * `json_texts` - A list of JSON texts.
    pub fn parse(&mut self, json_texts: &Vec<String>) -> Result<ModelDict, String> {
        let mut model_dict: ModelDict = ModelDict::new();

        self.add_primitive_schemas_to_model_dict(&mut model_dict)?;

        // Add the entries to the model dictionary for the primitive entity kinds.
        for entity_kind in EntityKind::iter() {
            if is_primitive_schema_kind(entity_kind) {
                let schema_info_id: Option<Dtmi> = create_dtmi(&entity_kind.to_string());
                if schema_info_id.is_none() {
                    return Err(format!(
                        "Cannot form a valid schema id for primitive entity kind '{entity_kind}."
                    ));
                }

                let boxed_entity_info = Box::new(PrimitiveSchemaInfoImpl::new(
                    DTDL_VERSION,
                    schema_info_id.clone().unwrap(),
                    None,
                    None,
                    entity_kind,
                ));
                model_dict.insert(schema_info_id.clone().unwrap(), boxed_entity_info);
            }
        }

        for json_text in json_texts {
            let mut doc: Value = match serde_json::from_str(json_text) {
                Ok(json) => json,
                Err(error) => {
                    return Err(format!("Failed to parse one of the JSON texts due to: {error}"))
                }
            };

            self.preprocess(&mut doc)?;

            let mut loader = NoLoader::<Value>::new();
            let dtdl_doc =
                block_on(doc.expand::<context::Json<Value>, _>(&mut loader)).map_err(|error| {
                    format!("Failed to expand one of the JSON texts due to: {error:?}")
                })?;

            for item in dtdl_doc.iter() {
                let object: &Object<serde_json::Value> = item;
                if let Object::Node(node) = object {
                    self.parse_node(node, &None, &mut model_dict)?;
                }
            }
        }

        Ok(model_dict)
    }

    /// Add the entries to the model dictionary for the primitive schemas.
    ///
    /// # Arguments
    /// * `model_dict` - The model dictionary.
    fn add_primitive_schemas_to_model_dict(
        &mut self,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        for entity_kind in EntityKind::iter() {
            if is_primitive_schema_kind(entity_kind) {
                let schema_info_id: Option<Dtmi> = create_dtmi(&entity_kind.to_string());
                if schema_info_id.is_none() {
                    return Err(format!(
                        "Cannot form a valid schema id for primitive schema {entity_kind}."
                    ));
                }

                let boxed_entity_info = Box::new(PrimitiveSchemaInfoImpl::new(
                    DTDL_VERSION,
                    schema_info_id.clone().unwrap(),
                    None,
                    None,
                    entity_kind,
                ));
                model_dict.insert(schema_info_id.clone().unwrap(), boxed_entity_info);
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
                    "Unable to parse the context located at {} due to: {:?}",
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
    /// * `context_name` - The name of the context that we want to replace.
    /// * 'context_value` - The JSON object that we will replace it with.
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
                    if let Value::String(_) = &a[i] {
                        if a[i] == context_name {
                            a[i] = context_value.clone();
                            break;
                        }
                    }
                }
            } else if let Value::Object(_o) = existing_context_value {
                // ignore - this one does not have an IRI associated with it.
            } else {
                return Err(format!("Unexpected context value '{existing_context_value:?}'"));
            }
        }
        Ok(())
    }

    /// Replace a name reference for a context in a JSON-LD document with its respective JSON value.
    ///
    /// # Arguments
    /// * `doc` - The JSON document.
    /// * `context_name` - The name of the context that we want to replace.
    /// * 'context_value` - The JSON object that we will replace it with.
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
    /// * `doc` - The JSON-LD document to preprocess.
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
        let dtdl_2_context_path_string = Self::find_full_path("v2/context/DTDL.v2.context.json")?;
        let dtdl_2_context_path_string_unwrapped = dtdl_2_context_path_string;
        let dtdl_2_context_path = Path::new(&dtdl_2_context_path_string_unwrapped);
        let dtdl_2_context_value = self.retrieve_context(dtdl_2_context_path)?;
        self.replace_context_inline_in_doc(doc, "dtmi:dtdl:context;2", &dtdl_2_context_value)?;

        Ok(())
    }

    /// Get a property value from a node by name.
    ///
    /// # Arguments
    /// * `node` - The node that contains the property.
    /// * `property_name` - The name of the property.
    fn get_property_value(
        &self,
        node: &Node<Value>,
        property_name: &str,
    ) -> Result<Option<String>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == property_name {
                if the_objects.len() == 1 {
                    match the_objects[0].as_str() {
                        Some(v) => return Ok(Some(String::from(v))),
                        None => {
                            return Err(String::from(
                                "get_property_value was unable to convert the value to a str",
                            ))
                        }
                    }
                } else {
                    return Err(String::from(
                        "get_property_value does not contain the expected number of objects",
                    ));
                }
            }
        }

        Ok(None)
    }

    /// Get the schema info for a primary or existing schema.  Both are represented by a schema name that could represent either case.
    /// This function will determine which one it is and return the corresponding schema info.
    ///
    /// # Arguments
    /// * `node` - The node that contains the schema's name.
    /// * `model_dict` - The model dictionary, containing the schema infos that have already been captured.
    /// * `parent_id` - The parent id.
    fn get_primary_or_existing_schema(
        &self,
        node: &Node<Value>,
        model_dict: &mut ModelDict,
        parent_id: &Option<Dtmi>,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        let string_option: Option<&str> = node.as_str();
        if let Some(schema_name) = string_option {
            let entity_kind_option: Option<EntityKind> = match EntityKind::from_str(schema_name) {
                Ok(v) => Some(v),
                Err(_) => None,
            };

            if let Some(entity_kind) = entity_kind_option {
                if is_primitive_schema_kind(entity_kind) {
                    let id: Option<Dtmi> = self.generate_id(parent_id, "test");
                    if id.is_none() {
                        return Err(String::from(
                            "we were not able to generate an id for the schema",
                        ));
                    }

                    Ok(Box::new(PrimitiveSchemaInfoImpl::new(
                        DTDL_VERSION,
                        id.unwrap(),
                        parent_id.clone(),
                        None,
                        entity_kind,
                    )))
                } else {
                    Err(format!("expected a primitive schema, found {entity_kind}"))
                }
            } else {
                self.retrieve_schema_info_from_model_dict(schema_name, model_dict)
            }
        } else {
            Err(String::from("get_schema encountered an unknown entity kind value"))
        }
    }

    /// Get an object schema info from a node.
    ///
    /// # Arguments
    /// * `node` - The node that contains the object schema's specification.
    /// * `model_dict` - The model dictionary, containing the schema infos that have already been captured.
    /// * `parent_id` - The parent id.
    fn get_object_schema(
        &self,
        node: &Node<Value>,
        model_dict: &mut ModelDict,
        parent_id: &Option<Dtmi>,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        let mut fields: Vec<Box<dyn FieldInfo>> = Vec::new();

        for (the_property, the_objects) in node.properties() {
            if the_property == "dtmi:dtdl:property:fields;2" {
                for i in 0..the_objects.len() {
                    if let Object::Node(node) = &*the_objects[i] {
                        let mut name_option: Option<String> = None;
                        let mut display_name_option: Option<String> = None;
                        let mut schema: Option<Box<dyn SchemaInfo>> = None;
                        for (the_property, the_objects) in node.properties() {
                            if the_property == "dtmi:dtdl:property:displayName;2"
                                && the_objects.len() == 1
                            {
                                if let Object::Value(value) = &*the_objects[0] {
                                    display_name_option = value.as_str().map(String::from)
                                }
                            } else if the_property == "dtmi:dtdl:property:schema;2"
                                && the_objects.len() == 1
                            {
                                if let Object::Node(node) = &*the_objects[0] {
                                    if node.properties().is_empty() {
                                        schema = Some(self.get_primary_or_existing_schema(
                                            node, model_dict, parent_id,
                                        )?);
                                    } else {
                                        schema = Some(
                                            self.get_complex_schema(node, model_dict, parent_id)?,
                                        );
                                    }
                                }
                            } else if the_property == "dtmi:dtdl:property:name;2"
                                && the_objects.len() == 1
                            {
                                if let Object::Value(value) = &*the_objects[0] {
                                    name_option = value.as_str().map(String::from)
                                }
                            }
                        }
                        if name_option.is_some() {
                            let id: Option<Dtmi> =
                                self.generate_id(parent_id, &name_option.clone().unwrap());
                            if id.is_none() {
                                return Err(String::from(
                                    "We were not able to generate an id for the schema.",
                                ));
                            }

                            let mut field_info = FieldInfoImpl::new(
                                DTDL_VERSION,
                                id.unwrap(),
                                parent_id.clone(),
                                None,
                                name_option,
                                schema,
                            );

                            field_info.set_display_name(display_name_option);

                            fields.push(Box::new(field_info));
                        }
                    }
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
            Some(fields),
        )))
    }

    /// Get a complex schema info from a node.
    ///
    /// # Arguments
    /// * `node` - The node that contains the complex schema's specification.
    /// * `model_dict` - The model dictionary, containing the schema infos that have already been captured.
    /// * `parent_id` - The parent id.
    fn get_complex_schema(
        &self,
        node: &Node<Value>,
        model_dict: &mut ModelDict,
        parent_id: &Option<Dtmi>,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        let mut entity_kind_option: Option<EntityKind> = None;
        for node_type in node.types() {
            let entity_kind_result = EntityKind::from_str(node_type.as_str());
            if let Ok(entity_kind) = entity_kind_result {
                entity_kind_option = Some(entity_kind);
                break;
            }
        }

        if entity_kind_option.is_none() {
            return Err(String::from("Complex schema has no associated type.  It must have one."));
        }

        let entity_kind = entity_kind_option.unwrap();

        if entity_kind == EntityKind::Object {
            self.get_object_schema(node, model_dict, parent_id)
        } else {
            Err(format!("Unsupported complex object: {entity_kind:?}."))
        }
    }

    /// Get a schema info from a node.
    ///
    /// # Arguments
    /// * `node` - The node that contains the schema's specification.
    /// * `model_dict` - The model dictionary, containing the schema infos that have already been captured.
    /// * `parent_id` - The parent id.
    fn get_schema(
        &self,
        node: &Node<Value>,
        model_dict: &mut ModelDict,
        parent_id: &Option<Dtmi>,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == "dtmi:dtdl:property:schema;2" {
                if the_objects.len() == 1 {
                    if let Object::Node(node) = &*the_objects[0] {
                        if node.properties().is_empty() {
                            return self
                                .get_primary_or_existing_schema(node, model_dict, parent_id);
                        } else {
                            return self.get_complex_schema(node, model_dict, parent_id);
                        }
                    } else {
                        return Err(String::from(
                            "The schema property's associated object should be a node.  It is not.",
                        ));
                    }
                } else {
                    return Err(format!(
                        "The schema property should only have 1 assoicated object.  It has {}.",
                        the_objects.len()
                    ));
                }
            }
        }

        Err(String::from("A schema property was not found."))
    }

    /// Get the payload.
    ///
    /// # Arguments
    /// * `node` - The node that contains the payload's specification.
    /// * `model_dict` - The model dictionary, containing the schema infos that have already been captured.
    /// * `property_name` - The property name associated with the payload.
    /// * `parent_id` - The parent id.
    fn get_payload(
        &self,
        node: &Node<Value>,
        model_dict: &mut ModelDict,
        property_name: &str,
        parent_id: &Option<Dtmi>,
    ) -> Result<Option<Box<dyn CommandPayloadInfo>>, String> {
        for (the_property, the_objects) in node.properties() {
            if the_property == property_name {
                if let Object::Node(node) = &*the_objects[0] {
                    // name - optional
                    let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

                    let mut id: Option<Dtmi> = None;
                    if node.id().is_some() {
                        id = create_dtmi(node.id().unwrap().as_str());
                    }
                    if id.is_none() {
                        if name.is_none() {
                            return Err(String::from(
                                "We cannot generate an id for the payload when we do not have a name.",
                            ));
                        }
                        id = self.generate_id(parent_id, &name.clone().unwrap());
                        if id.is_none() {
                            return Err(String::from(
                                "We were unable to generate an id for the payload.",
                            ));
                        }
                    }

                    // displayName - required
                    let _display_name =
                        self.get_property_value(node, "dtmi:dtdl:property:displayName;2")?;

                    // description - required
                    let _description =
                        self.get_property_value(node, "dtmi:dtdl:property:description;2")?;

                    // schema - required
                    let boxed_schema_info: Box<dyn SchemaInfo> =
                        self.get_schema(node, model_dict, &id)?;

                    return Ok(Some(Box::new(CommandPayloadInfoImpl::new(
                        DTDL_VERSION,
                        id.unwrap(),
                        parent_id.clone(),
                        None,
                        name,
                        Some(boxed_schema_info),
                    ))));
                } else {
                    return Err(String::from("get_payload encountered an unknown object"));
                }
            }
        }

        Ok(None)
    }

    /// Gather the undefined propeties from a node.
    ///
    /// # Arguments
    /// * `node` - The node to gather the undefined properties from.
    /// * `undefined_properties` - The resulting gathered undefined properties.
    fn gather_undefined_properties(
        node: &Node<Value>,
        undefined_properties: &mut HashMap<String, Value>,
    ) {
        for (the_property, the_objects) in node.properties() {
            if the_objects.len() == 1 {
                match &*the_objects[0] {
                    Object::Value(value) => {
                        let j = value.clone().as_json();
                        undefined_properties.insert(the_property.to_string(), j);
                    }
                    Object::Node(n) => {
                        Self::gather_undefined_properties(n, undefined_properties);
                    }
                    Object::List(_list) => {
                        warn!("gather_undefined_properties encountered a list");
                    }
                }
            }
        }
    }

    /// Genrate an id from the associated parent id and the associated property name.
    ///
    /// # Arguments
    /// * `parent_id` - The associated parent id.
    /// * `name` - The associated property name.
    fn generate_id(&self, parent_id: &Option<Dtmi>, name: &str) -> Option<Dtmi> {
        let generated_id_value = format!("{}:{}", parent_id.clone().unwrap().versionless(), name);
        create_dtmi(&generated_id_value)
    }

    /// Retrieve a schema info from a dictionary.
    ///
    /// # Arguments
    /// * `schema` - The id (as a string) for the schema info.
    /// * `model_dict` - The model dictionary to search.
    fn retrieve_schema_info_from_model_dict(
        &self,
        schema: &str,
        model_dict: &mut ModelDict,
    ) -> Result<Box<dyn SchemaInfo>, String> {
        let primitive_schema_info_id: Option<Dtmi> = create_dtmi(schema);
        if primitive_schema_info_id.is_none() {
            return Err(String::from("Primitive schema cannot form a valid schema id."));
        }
        let primitive_schema_info_model_entry =
            model_dict.get(&primitive_schema_info_id.clone().unwrap());
        if primitive_schema_info_model_entry.is_none() {
            return Err(format!(
                "We were not able to find the primitive schema entry for id '{}'.",
                primitive_schema_info_id.unwrap()
            ));
        }
        let boxed_primitive_schema_info_ref_result = primitive_schema_info_model_entry
            .unwrap()
            .as_any()
            .downcast_ref::<PrimitiveSchemaInfoImpl>();
        let boxed_schema_info: Box<dyn SchemaInfo> = match boxed_primitive_schema_info_ref_result {
            Some(boxed_primitive_schema_info_ref) => {
                Box::new((*boxed_primitive_schema_info_ref).clone())
            }
            None => return Err(String::from("Was not a primitive schema info")),
        };

        Ok(boxed_schema_info)
    }

    /// Retrieve an interface info from a model dictionary.
    ///
    /// # Arguments
    /// * `schema` - The id (as a string) for the interface info.
    /// * `model_dict` - The model dictionary to search.
    fn retrieve_interface_info_from_model_dict(
        &mut self,
        schema: &str,
        model_dict: &mut ModelDict,
    ) -> Result<Box<dyn InterfaceInfo>, String> {
        let interface_info_id: Option<Dtmi> = create_dtmi(schema);
        if interface_info_id.is_none() {
            return Err(String::from("Schema cannot form a valid schema id."));
        }
        let interface_info_model_entry = model_dict.get(&interface_info_id.clone().unwrap());
        if interface_info_model_entry.is_none() {
            return Err(format!(
                "We were not able to find the interface entry for id '{}'.",
                interface_info_id.unwrap()
            ));
        }
        let boxed_interface_schema_info_ref_result =
            interface_info_model_entry.unwrap().as_any().downcast_ref::<InterfaceInfoImpl>();
        let boxed_interface_info: Box<dyn InterfaceInfo> =
            match boxed_interface_schema_info_ref_result {
                Some(boxed_interface_schema_info_ref) => {
                    Box::new((*boxed_interface_schema_info_ref).clone())
                }
                None => return Err(String::from("Was not an interface info")),
            };

        Ok(boxed_interface_info)
    }

    /// Parse a node.
    ///
    /// # Arguments
    /// * `node` - The node to parse.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_node(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        let mut entity_kind_option: Option<EntityKind> = None;
        for node_type in node.types() {
            let entity_kind_result = EntityKind::from_str(node_type.as_str());
            if let Ok(entity_kind) = entity_kind_result {
                entity_kind_option = Some(entity_kind);
                break;
            }
        }

        if entity_kind_option.is_none() {
            return Err(String::from("Warning: No entity kind found amongst the node's types"));
        }

        match entity_kind_option.unwrap() {
            EntityKind::Interface => self.parse_interface(node, parent_id, model_dict)?,
            EntityKind::Telemetry => self.parse_telemetry(node, parent_id, model_dict)?,
            EntityKind::Property => self.parse_property(node, parent_id, model_dict)?,
            EntityKind::Command => self.parse_command(node, parent_id, model_dict)?,
            EntityKind::Relationship => self.parse_relationship(node, parent_id, model_dict)?,
            EntityKind::Component => self.parse_component(node, parent_id, model_dict)?,
            _ => return Err(String::from("Warning: Unexepcted entity kind found ")),
        }

        Ok(())
    }

    /// Parse an interface.
    ///
    /// # Arguments
    /// * `node` - The node that represents an interface.
    /// * `parent_id` - The interface's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_interface(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // @id - required
        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
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
                    self.parse_node(node, &id, model_dict)?;
                }
            }
        }

        // Add the interface to the model dictionary.
        let entity_info = Box::new(InterfaceInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
        ));
        model_dict.insert(id.clone().unwrap(), entity_info);

        Ok(())
    }

    /// Parse a telemetry.
    ///
    /// # Arguments
    /// * `node` - The node that represents a telemetry.
    /// * `parent_id` - The telemetry's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_telemetry(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // name - optional
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

        // schema - required
        let boxed_schema_info: Box<dyn SchemaInfo> =
            self.get_schema(node, model_dict, parent_id)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
        }
        if id.is_none() {
            if name.is_none() {
                return Err(String::from(
                    "We cannot generate an id for the telemtry when we do not have a name.",
                ));
            }
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the telemetry."));
            }
        }

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);

        let mut telemetry_info = TelemetryInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            name,
            Some(boxed_schema_info),
        );

        for (key, value) in undefined_property_values {
            telemetry_info.add_undefined_property(key, value);
        }

        model_dict.insert(id.unwrap(), Box::new(telemetry_info));

        Ok(())
    }

    /// Parse a property.
    ///
    /// # Arguments
    /// * `node` - The node that represents a property.
    /// * `parent_id` - The property's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_property(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // name - optional
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

        // schema - required
        let boxed_schema_info: Box<dyn SchemaInfo> =
            self.get_schema(node, model_dict, parent_id)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
        }
        if id.is_none() {
            if name.is_none() {
                return Err(String::from(
                    "We cannot generate an id for the property when we do not have a name.",
                ));
            }
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the property."));
            }
        }

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);

        let mut property_info = PropertyInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            name,
            Some(boxed_schema_info),
            false,
        );

        for (key, value) in undefined_property_values {
            property_info.add_undefined_property(key, value);
        }

        model_dict.insert(id.unwrap(), Box::new(property_info));

        Ok(())
    }

    /// Parse a command.
    ///
    /// # Arguments
    /// * `node` - The node that represents a command.
    /// * `parent_id` - The command's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_command(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // name - optional
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
        }
        if id.is_none() {
            if name.is_none() {
                return Err(String::from(
                    "We cannot generate an id for the command when we do not have a name.",
                ));
            }
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the command."));
            }
        }

        let request_payload: Option<Box<dyn CommandPayloadInfo>> =
            self.get_payload(node, model_dict, "dtmi:dtdl:property:request;2", &id)?;
        let response_payload: Option<Box<dyn CommandPayloadInfo>> =
            self.get_payload(node, model_dict, "dtmi:dtdl:property:response;2", &id)?;

        let mut undefined_property_values = HashMap::<String, Value>::new();
        Self::gather_undefined_properties(node, &mut undefined_property_values);

        let mut command_info = CommandInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            name,
            request_payload,
            response_payload,
        );

        for (key, value) in undefined_property_values {
            command_info.add_undefined_property(key, value);
        }

        model_dict.insert(id.clone().unwrap(), Box::new(command_info));

        Ok(())
    }

    /// Parse a relationship.
    ///
    /// # Arguments
    /// * `node` - The node that represents a relationship.
    /// * `parent_id` - The relationship's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_relationship(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // name - optional
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
        }
        if id.is_none() {
            if name.is_none() {
                return Err(String::from(
                    "We cannot generate an id for the relationship when we do not have a name.",
                ));
            }
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from(
                    "We were not able to generate an id for the relationship.",
                ));
            }
        }

        let entity_info = Box::new(RelationshipInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            name,
            None,
            false,
        ));
        model_dict.insert(id.unwrap(), entity_info);

        Ok(())
    }

    // Parse a component.
    ///
    /// # Arguments
    /// * `node` - The node that represents a component.
    /// * `parent_id` - The component's parent id.
    /// * `model_dict` - The model dictionary to add the content to.
    fn parse_component(
        &mut self,
        node: &Node<Value>,
        parent_id: &Option<Dtmi>,
        model_dict: &mut ModelDict,
    ) -> Result<(), String> {
        // name - optional
        let name = self.get_property_value(node, "dtmi:dtdl:property:name;2")?;

        // schema - required (note: here the schema property represents an interface)
        let schema = self.get_property_value(node, "dtmi:dtdl:property:schema;2")?;
        if schema.is_none() {
            return Err(String::from("Component does not have a schema property."));
        }
        let boxed_interface_info: Box<dyn InterfaceInfo> =
            self.retrieve_interface_info_from_model_dict(&schema.unwrap(), model_dict)?;

        let mut id: Option<Dtmi> = None;
        if node.id().is_some() {
            id = create_dtmi(node.id().unwrap().as_str());
        }
        if id.is_none() {
            if name.is_none() {
                return Err(String::from(
                    "We cannot generate an id for the component when we do not have a name.",
                ));
            }
            id = self.generate_id(parent_id, &name.clone().unwrap());
            if id.is_none() {
                return Err(String::from("We were not able to generate an id for the component."));
            }
        }

        let entity_info = Box::new(ComponentInfoImpl::new(
            DTDL_VERSION,
            id.clone().unwrap(),
            parent_id.clone(),
            None,
            name,
            Some(boxed_interface_info),
        ));
        model_dict.insert(id.unwrap(), entity_info);

        Ok(())
    }

    /// Find the full path given a relative path and a preset DTDL_PATH environment variable (containing a semicolon-separated list of DTDL directories).
    ///
    /// # Arguments
    /// `relative_path` - The relative path.
    pub fn find_full_path(relative_path: &str) -> Result<String, String> {
        match env::var(Self::DTDL_PATH) {
            Ok(paths) => {
                let split = paths.split(';');
                let vec: Vec<&str> = split.collect();
                for path in vec {
                    let full_path = Path::new(path).join(relative_path);
                    if full_path.exists() {
                        return Ok(full_path.to_str().unwrap().to_string());
                    }
                }
            }
            Err(_) => {
                return Err(String::from(
                    "Unable to get the environment variable DTDL_PATH. Please set it.",
                ))
            }
        }
        Err(String::from("Unable to resolve the full path"))
    }
}

#[cfg(test)]
mod model_parser_tests {
    use super::*;
    use log::trace;
    use std::fs;
    use std::path::Path;
    use std::vec::Vec;

    /// The DTDL-path environment variable name.
    const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

    /// Retrieve the contents of the DTDL from the specified file path.
    ///
    /// # Arguments:
    /// `file_path` - The file path where the DTDL is located.
    fn retrieve_dtdl(file_path: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        let read_result = fs::read_to_string(path);
        match read_result {
            Ok(contents) => Ok(contents),
            Err(error) => Err(format!("Unable to retrieve the DTDL due to: {error}")),
        }
    }

    /// Get the repository's directory.
    fn get_repo_dir() -> Option<String> {
        // CARGO_MANIFEST_DIR - The directory containing the manifest of your package.
        let cargo_manifest_dir_result = env::var(CARGO_MANIFEST_DIR);
        if let Ok(cargo_manifest_dir) = cargo_manifest_dir_result {
            let cargo_manifest_dir_path = Path::new(&cargo_manifest_dir);
            let parent_result = cargo_manifest_dir_path.parent();
            if let Some(parent) = parent_result {
                parent.to_str().map(String::from)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Set the DTDL_PATH environment, so that the tests can use it.
    fn set_dtdl_path() {
        let repo_dir_result = get_repo_dir();
        if let Some(repo_dir) = repo_dir_result {
            let value = format!(
                "{repo_dir}/external/opendigitaltwins-dtdl/DTDL;{repo_dir}/external/iot-plugandplay-models;{repo_dir}/dtdl-parser/dtdl;{repo_dir}/digital-twin-model/dtdl"
            );
            env::set_var(ModelParser::DTDL_PATH, &value);
            trace!("{}={value}", ModelParser::DTDL_PATH);
        } else {
            warn!(
                "Unable to set {}, as repo directory could not be determined.",
                ModelParser::DTDL_PATH
            );
        }
    }

    #[test]
    fn validation_test() {
        set_dtdl_path();

        let mut json_texts = Vec::<String>::new();

        let device_information_full_path_result =
            ModelParser::find_full_path("dtmi/azure/devicemanagement/deviceinformation-1.json");
        assert!(device_information_full_path_result.is_ok());
        let device_information_contents_result =
            retrieve_dtdl(&device_information_full_path_result.unwrap());
        assert!(device_information_contents_result.is_ok());
        json_texts.push(device_information_contents_result.unwrap());

        let thermostat_full_path_result = ModelParser::find_full_path("v2/samples/Thermostat.json");
        assert!(thermostat_full_path_result.is_ok());
        let thermostat_contents_result = retrieve_dtdl(&thermostat_full_path_result.unwrap());
        assert!(thermostat_contents_result.is_ok());
        json_texts.push(thermostat_contents_result.unwrap());

        let temp_controller_full_path_result =
            ModelParser::find_full_path("v2/samples/TemperatureController.json");
        assert!(temp_controller_full_path_result.is_ok());
        let temp_controller_contents_result =
            retrieve_dtdl(&temp_controller_full_path_result.unwrap());
        assert!(temp_controller_contents_result.is_ok());
        json_texts.push(temp_controller_contents_result.unwrap());

        let mut parser = ModelParser::new();
        let model_dict_result = parser.parse(&json_texts);
        assert!(
            model_dict_result.is_ok(),
            "parse failed due to: {}",
            model_dict_result.err().unwrap()
        );
        let model_dict = model_dict_result.unwrap();
        assert_eq!(
            model_dict.len(),
            31,
            "expected length was 31, actual length is {}",
            model_dict.len()
        );
    }

    #[test]
    fn find_full_path_test() {
        set_dtdl_path();

        let find_full_path_result = ModelParser::find_full_path("v3/content/sdv/hvac.json");
        assert!(find_full_path_result.is_ok());
        let full_path = find_full_path_result.unwrap();
        assert!(full_path.ends_with("/v3/content/sdv/hvac.json"));
    }
}
