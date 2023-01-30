// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

use crate::dtmi::Dtmi;
use crate::entity_kind::EntityKind;

pub trait EntityInfo: Any {
    /// Returns the DTDL version.
    fn dtdl_version(&self) -> i32;

    /// Returns the identifier of the DTDL element that corresponds to this object.
    fn id(&self) -> &Dtmi;

    /// Returns the kind of Entity, meaning the concrete DTDL type assigned to the corresponding element in the model.
    fn entity_kind(&self) -> EntityKind;

    /// Returns the identifier of the parent DTDL element in which this element is defined.
    fn child_of(&self) -> &Option<Dtmi>;

    /// Returns the identifier of the partition DTDL element in which this element is defined.
    fn defined_in(&self) -> &Option<Dtmi>;

    // Returns the description for this entity.
    fn description(&self) -> &Option<String>;

    // Returns the display name for this entity.
    fn display_name(&self) -> &Option<String>;

    /// Returns any undefined properties of the DTDL element that corresponds to this object.
    fn undefined_properties(&self) -> &HashMap<String, Value>;

    /// Add an undefined property.
    /// # Arguments
    /// * `key` - The property's name.
    /// * `value` - The property's value.
    // fn add_undefined_property(&mut self, key: String, value: Value);

    /// Returns the instance as an Any.
    fn as_any(&self) -> &dyn Any;
}
