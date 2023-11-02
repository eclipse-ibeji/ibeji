Tutorial on using DTDL to create an in-vehicle model
	- Provide references to DTDL tutorials for more information. Our tutorial, will not be a complete tutorial on using DTDL
	- Discuss about the subset of the DTDL we are using.
		○ For instance, if we are using only interfaces and properties then describe them.
		○ Be explicit as to how DTDL is used currently. From the samples, it appears I can use Ibeji without knowing DTDL.
		○ Basic concepts of DTDL related to the Ibeji samples may need to be explained.
	- Some fields may be considered optional in the DTDL schema, but we mandate that these fields be present
	- (Optional) Mention that we are based off VSS.
	- Mention that we are strict about using the DTMI IDs and discuss why.
	- Include a paragraph to highlight why DTDL is used.
	- Tutorial should explain "this is how we will use DTDL to put your own in-vehicle model"
	- The learning curve for DTDL is a concern for the Hackathon. Presumably, our hello world sample in our blueprint will consist of very simple DTDL
		○ Simple interfaces with a minimal set of properties.
No DTDL relationships.

# Tutorial: Create a vehicle model with DTDL

- [Introduction](#introduction)
- [1. Create a reference vehicle model with DTDL](#1-create-a-reference-vehicle-model-with-dtdl)
    - [1.1 DTDL Interfaces](#12-dtdl-properties)
    - [1.2 DTDL properties](#12-dtdl-properties)
    - [1.3 DTDL commands](#13-dtdl-commands)
- [2. DTDL validation](#2-dtdl-validation)
- [3. Translating DTDL to code](#3-translating-dtdl-to-code)

## Introduction

In this tutorial, you will learn how to create a vehicle model using [Digital Twins Definition Language (DTDL) version 3.0](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html), and build a corresponding Rust vehicle model. The focus will be on translating DTDL to a programming language, specifically Rust. While Rust is used here to be consistent with Ibeji's samples being in Rust, you can use any programming language.

This tutorial provides a basic understanding of DTDL in the context of our Ibeji samples, but it is not a comprehensive guide to DTDL. Specifically, it covers [interfaces](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface), [commands](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#command), [properties](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#property)

Please refer to the [DTDL v3](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html) documentation for more information on DTDL.

>Note: Currently in Ibeji, DTDL is used as a reference representation of the vehicle hardware. It serves as a guide to construct the vehicle model in code.

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

### 1.1 DTDL interfaces

>[DTDL v3 Interface:](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface) "An Interface describes the contents (Commands, Components, Properties, Relationships, and Telemetries) of any digital twin".

Each sample DTDL file in the {repo-root-dir}/digital-twin-model/dtdl directory, begins with an interface at the top level.

A suggested strategy for creating your in-vehicle digital twin model is to first determine the components and vehicle characteristics you want to include, followed by deciding the level of granularity you need for your in-vehicle digital twin model.

For example, in our samples, we categorize HVAC and OBD separately. The `hvac.json` DTDL file contains all HVAC-related elements, while the `obd.json` DTDL file encompasses all OBD-related components.

Let's consider the `hvac.json` DTDL file:
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


Please see the [Interface](https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html#interface) section for the descriptions on each field.




Please ignore the JSON objects inside the contents field for now. This will be discussed in [1.2 DTDL properties](#12-dtdl-properties).

### 1.2 DTDL properties

### 1.3 DTDL commands

## 2. DTDL validation

## 3. Translating DTDL to code