# Digital Twin Model

## Specifying the Digital Twin Model

The digital twin model is specified using the Digital Twin Definition Language ([DTDL V3](https://github.com/Azure/opendigitaltwins-dtdl/blob/master/DTDL/v3/DTDL.v3.md)). The DTDL is stored under the `digital-twin-model/dtdl` folder and each file defines a DTDL interface.

DTDL's behavior is defined by several metamodel classes. Initially, we will only use the Interface, Property, Command and Relationship metamodel classes.

## DTDL Instance Representation in JSON-LD

Instances of DTDL models are represented as JSON-LD objects. Each instance is a JSON object that includes the `@context`, `@id`, and `@type` fields, as well as any properties defined in the model.

The `@context` field provides the context required for interpreting the JSON-LD document. The `@id` field is a unique identifier for the instance, and the `@type` field specifies the DTDL model that the instance is based on. The properties of the instance are represented as key-value pairs in the JSON object.

Here's an example of a DTDL instance represented in JSON-LD:

```json
{
  "@context": "dtmi:dtdl:context;3",
  "@id": "dtmi:com:example:Thermostat:device1;1",  // Unique identifier for the instance
  "@type": "dtmi:com:example:Thermostat;1",  // The DTDL model that the instance is based on
  "Temperature": 20.0,  // Property defined in the model
  "TargetTemperature": 22.0,  // Property defined in the model
  "MaxTempSinceLastReset": 25.0  // Property defined in the model
}
```

In this example, the `@context` field specifies the version of DTDL being used. For DTDL version 3, the appropriate context specifier is `dtmi:dtdl:context;3`. The `@id` field is a unique identifier for the thermostat device instance, and the `@type` field specifies that the instance is based on the `dtmi:com:example:Thermostat;1` model. The `Temperature`, `TargetTemperature`, and `MaxTempSinceLastReset` fields are properties defined in the model, and their values are set for this specific instance.

In DTDL, a **Relationship** is a directional link from a source digital twin to a target digital twin. Relationships are used to create and navigate the graph of digital twins.

Here's an example of how a relationship might be represented in a DTDL model:

```json
{
  "@context": "dtmi:dtdl:context;3",
  "@id": "dtmi:com:example:Building;1",
  "@type": "Interface",
  "displayName": "Building",
  "contents": [
    {
      "@type": "Relationship",
      "name": "hasFloor",
      "minMultiplicity": 0,
      "maxMultiplicity": 100,
      "target": "dtmi:com:example:Floor;1"
    }
  ]
}
```

In this example, the `hasFloor` relationship represents a link from a `Building` instance to a `Floor` instance. The `minMultiplicity` and `maxMultiplicity` fields specify that a `Building` can have between 0 and 100 `Floor` instances. The `target` field specifies the DTDL model that the target of the relationship is based on.

Here's an example of how a relationship might be represented in a DTDL instance:

```json
{
  "@context": "dtmi:dtdl:context;3",
  "@id": "dtmi:com:example:Building:building1;1",  // Unique identifier for the instance
  "@type": "dtmi:com:example:Building;1",  // The DTDL model that the instance is based on
  "hasFloor": [
    "dtmi:com:example:Floor:floor1;1",
    "dtmi:com:example:Floor:floor2;1"
  ]
}
```

In this example, the `hasFloor` field is an array of identifiers for `Floor` instances that are related to the `Building` instance.

## Using the Digital Twin Model in the Code

The digital twin providers and consumers need to reference the Digital Twin Model to perform digital twin interactions. They need the following information from the model:

- the model identifiers (denoted by "@id" in the DTDL) to identify the part of the model that they want to use
- the model names (denoted by "name" in the DTDL) to identify the member of a model part that they want to use
- the property definitions to exchange the associated values
- the command definitions to send and receive the appropriate payloads.

We need to make the digital twin model accessible as code. Each programming language will need its own variant. In this code repository, we will
provide a Rust variant. We will place all of the code for the model's content in a single file named after the model and its version. In
this code repository, that file will be the 'digital-twin-model/src/sdv_v1.rs' file.

The 'sdv_v1.rs' file is based on the model content from the DTDL files located under 'digital-twin-model/dtdl/dtmi/sdv'. A legacy version of this file 'sdv_v0.rs' is also present; it remains to support some of the older samples.

The 'sdv_v1.rs' file will setup namespaces based on the DTMIs used in the DTDL files. These namespaces will allow developers to provide fully
qualified names that look similar to the DTMIs (minus the 'dtmi:sdv:' prefix).

For each namespace, we will define:

- Constants for model ids (“ID”), model descriptions (“DESCRIPTION”), and model member names (“NAME”).
- Structs named “TYPE” to define property values, command request payloads, command response payloads, and schema types.
