// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

/// Indicates the kind of Entity, meaning the concrete DTDL type assigned to the corresponding element in the model.
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, Display)]
pub enum EntityKind {
    #[strum(serialize = "dtmi:dtdl:class:Array;2")]
    Array,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:boolean;2")]
    Boolean,

    #[strum(serialize = "dtmi:dtdl:class:Command;2")]
    Command,

    #[strum(serialize = "dtmi:dtdl:class:CommandPayload;2")]
    CommandPayload,

    #[strum(serialize = "dtmi:dtdl:class:CommandType;2")]
    CommandType,

    #[strum(serialize = "dtmi:dtdl:class:Component;2")]
    Component,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:date;2")]
    Date,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:dateTime;2")]
    DateTime,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:double;2")]
    Double,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:duration;2")]
    Duration,

    #[strum(serialize = "dtmi:dtdl:class:Enum;2")]
    Enum,

    #[strum(serialize = "dtmi:dtdl:class:EnumValue;2")]
    EnumValue,

    #[strum(serialize = "dtmi:dtdl:class:Field;2")]
    Field,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:float;2")]
    Float,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:integer;2")]
    Integer,

    #[strum(serialize = "dtmi:dtdl:class:Interface;2")]
    Interface,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:long;2")]
    Long,

    #[strum(serialize = "dtmi:dtdl:class:Map;2")]
    Map,

    #[strum(serialize = "dtmi:dtdl:class:MapKey;2")]
    MapKey,

    #[strum(serialize = "dtmi:dtdl:class:MapValue;2")]
    MapValue,

    #[strum(serialize = "dtmi:dtdl:class:Object;2")]
    Object,

    #[strum(serialize = "dtmi:dtdl:class:Property;2")]
    Property,

    #[strum(serialize = "dtmi:dtdl:class:Relationship;2")]
    Relationship,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:string;2")]
    String,

    #[strum(serialize = "dtmi:dtdl:class:Telemetry;2")]
    Telemetry,

    #[strum(serialize = "dtmi:dtdl:instance:Schema:time;2")]
    Time,

    #[strum(serialize = "dtmi:dtdl:class:Unit;2")]
    Unit,

    #[strum(serialize = "dtmi:dtdl:class:UnitAttribute;2")]
    UnitAttribute,

    #[strum(serialize = "dtmi:dtdl:class:CommandRequest;2")]
    CommandRequest,

    #[strum(serialize = "dtmi:dtdl:class:CommandResponse;2")]
    CommandResponse,

    #[strum(serialize = "dtmi:dtdl:class:LatentType;2")]
    LatentType,

    #[strum(serialize = "dtmi:dtdl:class:NamedLatentType;2")]
    NamedLatentType,

    #[strum(serialize = "dtmi:dtdl:class:Reference;2")]
    Reference,
}
