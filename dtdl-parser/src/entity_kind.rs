// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use strum_macros::EnumString;

/// Indicates the kind of Entity, meaning the concrete DTDL type assigned to the corresponding element in the model.
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString)]
pub enum EntityKind {
    #[strum(serialize = "dtmi:dtdl:class:Interface;2")]
    Interface,

    #[strum(serialize = "dtmi:dtdl:class:Telemetry;2")]
    Telemetry,

    #[strum(serialize = "dtmi:dtdl:class:Property;2")]
    Property,

    #[strum(serialize = "dtmi:dtdl:class:Command;2")]
    Command,

    #[strum(serialize = "dtmi:dtdl:class:Relationship;2")]
    Relationship,

    #[strum(serialize = "dtmi:dtdl:class:Component;2")]
    Component,
}
