# Tutorial: Create a Provider

- [Introduction](#introduction)
- [Prerequisites](#introduction)
- [1. Create an Ibeji Digital Twin Provider](#1-create-an-ibeji-digital-twin-provider)
  - [1.1 Define Digital Twin Provider Interface](#11-define-digital-twin-provider-interface)
  - [1.2 Provider Implementation](#12-create-hvac-and-hmi-interfaces)
- [2. Register with the In-Vehicle Digital Twin Service](#2-dtdl-validation)
- [3. (Optional) Enable Managed Subscribe](#2-dtdl-validation)
- [Next Steps](#next-steps)

## Introduction

>[Digital Twin Provider:](https://github.com/eclipse-ibeji/ibeji#high-level-design) "A provider exposes a subset of the vehicle's primary capabilities by registering them with the In-Vehicle Digital Twin Service. Once registered with the In-Vehicle Digital Twin Service they can in turn be offered to Ibeji consumers. Each capability includes meta data that allow Ibeji consumers to comprehend the nature of the capability, how to work with it and how it can be remotely accessed".

In this tutorial, you will leverage your in-vehicle model in code that you have created in the previous tutorial in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md) to create a digital twin provider. Additionally, you will learn how to register your digital twin provider with the [In-Vehicle Digital Twin Service](../../design/README.md#in-vehicle-digital-twin-service).

This tutorial will reference the sample code provided in Ibeji to keep the tutorial concise. Relevant code snippets are explicitly highlighted and discussed to ensure a clear understanding of the concepts.

In this tutorial, we be focusing on these specific parts of the code (provide links or snippets). While the full sample code (link to sample code) contains additional functionality, those parts are not directly relevant to this tutorial.

## Prerequisites

- Complete the tutorial in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md).
- Basic knowledge about [Protocol Buffers version 3.0](https://protobuf.dev/programming-guides/proto3/).
- Basic knowledge about the [gRPC protocol](https://grpc.io/docs/what-is-grpc/introduction/).

## 1. Create an Ibeji Digital Twin Provider

In this section, you will learn how to develop a digital twin provider that communicates with its consumers via gRPC. It is important to note that digital twin providers in Ibeji are protocol-agnostic. This means they are not restricted to using gRPC and can employ other communication protocols.

The `{repo-root-dir}/samples/mixed` directory contains code for the sample digital twin provider used in this tutorial. The `{repo-root-dir}/digital-twin-model/src` contains the in-vehicle model in Rust code that you have constructed in the previous [Tutorial: Create an In-vehicle Model With DTDL](../in_vehicle_model/README.md) along with additional signals not needed for this tutorial.

For simplicity, we will refer to both the `{repo-root-dir}/samples/mixed` and `{repo-root-dir}/digital-twin-model/src` directories throughout this tutorial.

### 1.1 Define Digital Twin Provider Interface

Ibeji

Each sample DTDL file in the `{repo-root-dir}/digital-twin-model/dtdl` directory begins with an interface at the top level.

>Tip: A suggested strategy for defining your digital twin provider is to first determine the components and in-vehicle characteristics you want to include in your common in-vehicle, followed by deciding the level of granularity you need for your in-vehicle digital twin model. For example, in our samples, we categorize HVAC and OBD separately. The `hvac.json` DTDL file contains all HVAC-related elements, while the `obd.json` DTDL file contains all OBD-related components.

### 1.2 Provider Implementation

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

Please see the [Interface](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface) section for the descriptions on each field and the required DTDL fields.

In addition to `@context`, `@type`, and `@id` fields, Ibeji requires you to include the `description` field. The `description` field is useful as it offers extra metadata for DTDL file labeling and logging.

The `contents` field will be discussed further in the [1.4 DTDL Properties](#14-dtdl-properties) and [1.5 DTDL Commands](#15-dtdl-commands) sections.

## 2. Register with the In-Vehicle Digital Twin Service

Ensure your digital twin models adhere to the [DTDL v3 spec](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3) before translating your digital twin models to code. Use Ibeji's [DTDL tools](https://github.com/eclipse-ibeji/ibeji/blob/main/dtdl-tools/README.md) to validate your DTDL files.

## 3. (Optional) Enable Managed Subscribe

You have built a basic in-vehicle digital twin model with HVAC  and HMI systems using DTDL. In this section, you will convert this model into Rust code.

1. Create a Rust file called `sdv_v1.rs`. This file will provide metadata from the in-vehicle digital twin model.

1. Reference the `hmi.json` DTDL file that you have created in [1.5 DTDL Commands](#15-dtdl-commands).

1. Copy the following contents to the `sdv_v1.rs`:

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
```

1. Reference the `hvac.json` DTDL file that you have created in [1.4 DTDL Properties](#14-dtdl-properties)

1. Copy the following contents to the `sdv_v1.rs` Rust file:

```rust
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

This `sdv_v1.rs` is a representation of the in-vehicle DTDL model with an HMI and an HVAC in code.

In [1.2 Create HVAC and HMI interfaces](#12-create-hvac-and-hmi-interfaces), the `@id` field for the HMI and HVAC digital twin interfaces are `dtmi:sdv:HMI;1` and `dtmi:sdv:HVAC;1`, respectively. These DTMIs are constructed in the `sdv_v1.rs` file, which we created in step 3 and step 5.

The `sdv_v1.rs` file contains two main modules: `hmi` and `hvac`. The `hmi` module constructs the interface for `dtmi:sdv:HMI;1`, and the `hvac` module constructs the interface for `dtmi:sdv:HVAC;1`.

In the `hmi` module, there is a `show_notification` submodule that represents the `ShowNotification` command in DTDL. It has an `ID`, `NAME` and `DESCRIPTION` constants, which correspond to the `@id`, `name`, and `description` fields in the `hmi.json` DTDL file. The request and response submodules represent the request and response of the command.

Similarly, in the `hvac` module, there are two submodules: `ambient_air_temperature` and `is_air_conditioning_active`. These represent the `AmbientAirTemperature` and `IsAirConditioningActive` properties in the `hvac.json` DTDL. Each submodule has an `ID`, `NAME`, `DESCRIPTION` and `TYPE` constants, which correspond to the `@id`, `name`, `description`, and `schema` fields in DTDL.

This Rust code is a way to use a DTDL model in a Rust program, with each DTDL element represented as a Rust module, constant, or type. You can translate a DTDL model into other programming languages. Use the `@id` fields in your in-vehicle digital twin model as guidance to code your in-vehicle model.

Both Ibeji providers and Ibeji consumers can utilize this code. This code serves as a set of constants to standardize the values used in their communication with Ibeji, which ensures a consistent and reliable exchange of information.
