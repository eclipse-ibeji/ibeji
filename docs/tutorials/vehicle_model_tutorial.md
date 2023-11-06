# Tutorial: Create a vehicle model with DTDL

- [Introduction](#introduction)
- [1. Create a reference vehicle model with DTDL](#1-create-a-reference-vehicle-model-with-dtdl)
    - [1.1 DTDL Interfaces](#11-dtdl-interfaces)
    - [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces)
    - [1.3 Required Fields](#13-required-fields)
    - [1.4 DTDL DTMI ID](#14-dtdl-dtmi-id)
    - [1.5 DTDL properties](#15-dtdl-properties)
    - [1.6 DTDL commands](#16-dtdl-commands)
- [2. DTDL validation](#2-dtdl-validation)
- [3. Translating DTDL to code](#3-translating-dtdl-to-code)

## Introduction

In this tutorial, you will learn how to create a vehicle model using [Digital Twins Definition Language (DTDL) version 3.0](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html), and build a corresponding Rust vehicle model. The focus will be on translating DTDL to a programming language, specifically Rust. While Rust is used here to be consistent with Ibeji's samples being in Rust, you can use any programming language.

This tutorial provides a basic understanding of DTDL in the context of our Ibeji samples, but it is not a comprehensive guide to DTDL. Specifically, it covers [interfaces](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface), [commands](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#command), [properties](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#property)

Please refer to the [DTDL v3](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html) documentation for more information on DTDL.

>Note: Currently in Ibeji, DTDL is used as a reference representation of the vehicle hardware. It serves as a guide to manually construct the vehicle model in code. In the future, the vehicle model code should be generated from a DTDL file.

## 1. Create a reference vehicle model with DTDL

In this section, you will be learning how to create a reference vehicle model with DTDL. DTDL is used to define a digital twin model of a vehicle, and its capabilities.

Under the {repo-root-dir}/digital-twin-model/dtdl directory, contains sample DTDL
files to describe our sample in-vehicle digital twin model:
```
dtdl
└── v3
    └── spec
        └── sdv
            ├── airbag_seat_massager.json
            ├── camera.json
            ├── hmi.json
            ├── hvac.json
            └── obd.json
```

Our sample in-vehicle digital twin model consists of an airbag seat massager, camera, human machine interface (HMI), HVAC, and an on-board diagnostics (OBD).

To simplify, this tutorial will guide you in creating a digital twin vehicle model with HVAC and HMI systems.

### 1.1 DTDL interfaces

>[DTDL v3 Interface:](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface) "An Interface describes the contents (Commands, Components, Properties, Relationships, and Telemetries) of any digital twin".

Each sample DTDL file in the {repo-root-dir}/digital-twin-model/dtdl directory, and begins with an interface at the top level.

>Tip: A suggested strategy for creating your in-vehicle digital twin model is to first determine the components and vehicle characteristics you want to include in your common vehicle, followed by deciding the level of granularity you need for your in-vehicle digital twin model.
>
>For example, in our samples, we categorize HVAC and OBD separately. The `hvac.json` DTDL file contains all HVAC-related elements, while the `obd.json` DTDL file encompasses all OBD-related components.

### 1.2 Create HVAC and HMI interfaces

1. Create a file named `hvac.json`
1. Create the DTDL interface for HVAC:
  ```json
  [
    {
      "@context": ["dtmi:dtdl:context;3"],
      "@type": "Interface",
      "@id": "dtmi:sdv:HVAC;1",
      "description": "Heat, Ventilation and Air Conditioning",
      "contents": []
    }
  ]
  ```
1. Create a file named `hmi.json`
1. Create the DTDL interface for HMI:
  ```json
  [
    {
      "@context": ["dtmi:dtdl:context;3"],
      "@type": "Interface",
      "@id": "dtmi:sdv:HMI;1",
      "description": "The Human Machine Interface.",
      "contents": []
    }
  ]
  ```

### 1.3 Required Fields

Please see the [Interface](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface) section for the descriptions on each field and the required DTDL fields.

In addition to `@context`, `@type`, and `@id` fields, Ibeji requires you to include the `description` field. The `description` field is useful as it offers extra metadata for DTDL file labeling and information logging.

The `contents` field will be discussed further in the [1.5 DTDL properties](#15-dtdl-properties) and [1.6 DTDL commands](#16-dtdl-commands-dtdl-commands) sections.

### 1.4 DTDL DTMI ID

>[Digital Twin Model Identifer (DTMI):](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#additional-concerns) "All elements in digital twin models must have an identifier that is a digital twin model identifier (DTMI). This includes Interfaces, Properties, Telemetries, Commands, Relationships, Components, complex schema objects, etc. This does not require that every model element have an explicit identifier, but any identifier assigned to a model element by a digital twin processor must follow this identifier format".

The value for the `@id` field must conform to the DTMI format. A DTMI consists of three parts: scheme, path, and version. They are arranged in the format: `<scheme>:<path>;<version>`, with a colon separating the scheme and path, and a semicolon dividing the path and version. A DTMI must be unique. Please reference the [Digital Twin Mode Identifier](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#digital-twin-model-identifier) section of the DTDL document to learn more about DTMI.

The suggested approach for your creating a [DTMI](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#digital-twin-model-identifier): "For any definition that is the property of an organization with a registered domain name, a suggested approach to generating identifiers is to use the reversed order of domain segments as initial path segments, followed by further segments that are expected to be collectively unique among definitions within the domain. For example, dtmi:com:fabrikam:industrialProducts:airQualitySensor;1".

In step 2 of [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces), the DTMI for your HVAC system is `dtmi:sdv:HVAC;1` and `dtmi:sdv:HMI;1` for your HMI. The domain is `sdv` and under this domain we have an `HVAC` and an `HMI` interface. These two DTMIs conforms to the DTDL v3 spec.

### 1.5 DTDL properties

>[Property:](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3#property) "A Property describes the read-only and read/write state of any digital twin. For example, a device serial number may be a read-only Property; the desired temperature on a thermostat may be a read-write Property; and the name of a room may be a read-write Property".

Consider our HVAC digital twin that you created in [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces).

To add properties to the HVAC digital twin model, replace the existing content of `hvac.json` with the following:

```json
[
  {
    "@context": ["dtmi:dtdl:context;3"],
    "@type": "Interface",
    "@id": "dtmi:sdv:HVAC;1",
    "description": "Heat, Ventilation and Air Conditioning",
    "contents": [
      {
          "@type": "Property",
          "@id": "dtmi:sdv:HVAC:AmbientAirTemperature;1",
          "name": "AmbientAirTemperature",
          "description": "The immediate surroundings' air temperature (in Fahrenheit).",
          "schema": "integer"
        },
        {
          "@type": "Property",
          "@id": "dtmi:sdv:HVAC:IsAirConditioningActive;1",
          "name": "IsAirConditioningActive",
          "description": "Is air conditioning active?",
          "schema": "boolean"
        }
    ]
  }
]
```

This introduces two signals to our HVAC system: ambient air temperature and air conditioning status. Both signals have `@id` values starting with `dtmi:sdv:HVAC`, signifying they belong to the sdv domain and HVAC interface.

Please see [Property](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3#property) for more information on the property type and the descriptions of each field. Similar to the DTDL interface type, Ibeji mandates the description field. Despite DTDL v3 spec considering the @id field for properties as optional, Ibeji requires it. This helps in translating your DTDL files to code.

You can add more signals to the HVAC system, but ensure they are properties, not commands, which we will discuss in the next section. Signals unrelated to HVAC should not be included in the HVAC interface. As suggested in [1.1 DTDL interfaces](#11-dtdl-interfaces), it is beneficial to segregate interfaces to maintain conciseness and group related components together.

### 1.6 DTDL commands

>[Command:](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3#command) "A Command describes a function or operation that can be performed on any digital twin".

Consider our HVAC digital twin that you created in [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces).

To add properties to the HMI digital twin model, replace the existing content of `hmi.json` with the following:

```json
[
  {
    "@context": ["dtmi:dtdl:context;3"],
    "@type": "Interface",
    "@id": "dtmi:sdv:HMI;1",
    "description": "The Human Machine Interface.",
    "contents": [
      {
        "@type": "Command",
        "@id": "dtmi:sdv:HMI:ShowNotification;1",
        "name": "ShowNotification",
        "description": "Show a notification on the HMI.",
        "request": {
          "@id": "dtmi:sdv:HMI:ShowNotification:request;1",
          "name": "Notification",
          "displayName": "Notification",
          "description": "The notification to show on the HMI.",
          "schema": "string"
        }
      }
    ]
  }
]
```

This introduces a command to our HMI system: Show Notification. Similar to the properties we introduced in the previous section, this command has an `@id` value starting with `dtmi:sdv:HMI`, signifying that this command belongs to the sdv domain and HMI interface.

The `ShowNotification` is not a property. This is because properties reflect the state of a digital twin, while `ShowNotification` is an action that can be performed on this HMI digital twin model which qualifies as a command.

Please see [Command](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3#command) for more information on the command type and the descriptions of each field. Similar to the DTDL interface type, Ibeji mandates the description field. Despite DTDL v3 spec considering the `@id` field for commands as optional, Ibeji requires it. This helps in translating your DTDL files to code.

## 2. DTDL validation

Ensure your digital twin models adhere to the [DTDL v3 spec](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3) before translating your digital twin models to code. Use Ibeji's [DTDL tools](https://github.com/eclipse-ibeji/ibeji/blob/main/dtdl-tools/README.md) to validate your DTDL files.

## 3. Translating DTDL to code

You have finally finished constructing your reference simple vehicle model using DTDL. Your reference vehicle model now consists of an HVAC and an HMI. In this section, you will use your reference vehicle model and translate it to Rust code.

1. Reference the `hvac.json` and `hmi.json` you created in [1.5 DTDL properties](#15-dtdl-properties) and [1.6 DTDL commands](#16-dtdl-commands).

1. Create a Rust file called `sdv_v1.rs`, and copy the following contents:

```rust
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Note: In the future this code should be generated from a DTDL spec.

pub mod hmi {
    pub mod show_notification {
        pub const ID: &str = "dtmi:sdv:HMI:ShowNotification;1";
        pub const NAME: &str = "Show Notification";
        pub const DESCRIPTION: &str = "Show a notification on the HMI.";
        pub mod request {
            pub const ID: &str = "dtmi:sdv:HMI:ShowNotification::request;1";
            pub const NAME: &str = "Notification";
            pub const DESCRIPTION: &str = "The notification to show on the HMI.";
            pub type TYPE = String;
        }
        pub mod response {
            pub const ID: &str = "dtmi:sdv:HMI:ShowNotification::response;1";
        }
    }
}

pub mod hvac {
    pub mod ambient_air_temperature {
        pub const ID: &str = "dtmi:sdv:HVAC:AmbientAirTemperature;1";
        pub const NAME: &str = "AmbientAirTemperature";
        pub const DESCRIPTION: &str = "The immediate surroundings air temperature (in Fahrenheit).";
        pub type TYPE = i32;
    }

    pub mod is_air_conditioning_active {
        pub const ID: &str = "dtmi:sdv:HVAC:IsAirConditioningActive;1";
        pub const NAME: &str = "IsAirConditioningActive";
        pub const DESCRIPTION: &str = "Is air conditioning active?";
        pub type TYPE = bool;
    }
}
```

This Rust code is a representation of the vehicle DTDL model that has an HMI and an HVAC.

In [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces), the `@id` field for the HMI and HVAC digital twin interfaces are `dtmi:sdv:HMI;1` and `dtmi:sdv:HVAC;1`, respectively. These DTMIs are constructed in the sdv_v1.rs file, which we created in step 2.

The sdv_v1.rs file contains two main modules: `hmi` and `hvac`. The HMI module constructs the interface for `dtmi:sdv:HMI;1`, and the hvac module constructs the interface for `dtmi:sdv:HVAC;1`.

In the HMI module, there is a `show_notification` submodule that represents the ShowNotification command in DTDL. It has an ID, NAME, and DESCRIPTION constant, which correspond to the `@id`, `name`, and `description` fields in the `hmi.json` DTDL file. The request and response submodules represent the request and response of the command.

Similarly, in the HVAC module, there are two submodules: ambient_air_temperature and is_air_conditioning_active. These represent the `AmbientAirTemperature` and `IsAirConditioningActive` properties in the `hvac.json` DTDL. Each submodule has an ID, NAME, DESCRIPTION, and TYPE constant, which correspond to the `@id`, `name`, `description`, and `schema` fields in DTDL.

This Rust code is a way to use a DTDL model in a Rust program, with each DTDL element represented as a Rust constant or type. You can translate a DTDL model into other programming languages. Use the `@id` fields in your DTDL model as guidance to code your vehicle model.