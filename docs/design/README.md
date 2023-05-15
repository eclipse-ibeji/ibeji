# Design Specification for Project Eclipse Ibeji

- [Introduction](#introduction)
- [Architecture](#architecture)
- [DTDL](#dtdl)
- [In-Vehicle Digital Twin Service](#in-vehicle-digital-twin-service)
- [Provider](#provider)
- [Consumer](#consumer)

## <a name="introduction">Introduction</a>

Project Eclipse Ibeji delivers an In-Vehicle software component that is a digital representation of vehicle hardware resources. The representation is usable by other software in the vehicle to monitor and control vehicle hardware resources in a standardized manner.

Please note that the initial Ibeji implementation is a proof-of-concept. We would like to see it evolve into an enterprise class solution.

## <a name="architecture">Architecture</a>

Ibeji has three main architectural concepts:

- Digital Twin Consumer
- Digital Twin Provider
- In-Vehicle Digital Twin Service

The first Ibeji architectural concept that we will introduce is the Digital Twin Consumer. A Digital Twin Consumer is a software entity that utilizes Ibeji to interface with the digital representation of the In-Vehicle hardware components.

Another Ibeji architectural concept is the Digital Twin Provider. A Digital Twin Provider is the access point to some/all of the vehicle's hardware resources. A Digital Twin Provider registers itself with the In-Vehicle Digital Twin Service. Once registered, the In-Vehicle Digital Twin Service can make the resources available to Digital Twin Consumers. Each resource includes meta data so that DIgital Twin Consumers know how to interact with it. The In-Vehicle Digital Twin Service supports multiple simultaneous Digital Twin Providers and internally resolves overlapping resources offered by multiple Digital Twin Providers. These overlaps offer multiple options for interacting with a resource and can improve the resource's availability (by supporting multiple access paths).

In the middle is the In-Vehicle Digital Twin Service. It exports a query interface that enables Digital Twin Consumers to discover the vehicle's resources and provides the details necessary to use those resources. The In-Vehicle Digital Twin Service has an interface that allows Digital Twin Providers to dynamically register their resources.

Below is the component diagram for Ibeji.

![Component Diagram](diagrams/ibeji_component.svg)

## <a name="dtdl">DTDL</a>

Fundamental to the Ibeji solution is its use of Digital Twin Definition Language [DTDL](https://github.com/Azure/opendigitaltwins-dtdl) to identify and specify each of the vehicle's resources.

This initial contribution does not try to arrange the resources into a hierarchy or into a graph. It is intended that some future update will enable this capability.

DTDL can identify and specify each of the resources. Below is an example for the AmbientAirTemperature property.

```uml
  {
    "@context": ["dtmi:dtdl:context;2"],
    "@type": "Interface",
    "@id": "dtmi:sdv:Vehicle:Cabin:HVAC;1",
    "contents": [
      {
        "@type": ["Property", "Temperature"],
        "@id": "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1",
        "name": "Cabin_AmbientAirTemperature",
        "description": "The immediate surroundings air temperature (in Fahrenheit).",
        "schema": "integer",
        "unit": "degreeFahrenheit"
      }
    ]
  }
```

The DTDL must use the standard dtmi dtdl context.

## <a name="in-vehicle-digital-twin-service">In-Vehicle Digital Twin Service</a>

### In-Vehicle Digital Twin Service Overview

The initial In-Vehicle Digital Twin Service will provide the functionality needed by the proof-of-concept. On the Provider side, this initial contribution supports only a single Provider registering its DTDL. On the Consumer side, there is a simplified query api, and the ability to subscribe to a provided hardware resource data feed and to invoke a command on a provided hardware resource.

### Interfaces

The initial In-Vehicle Digital Twin Service supports both Providers and Consumers.

### Activities

#### Register

Below is the sequence diagram for the Register activity.

![Sequence Diagram](diagrams/register_sequence.svg)

#### Find by Id

Below is the sequence diagram for the Find-By-Id activity.

![Sequence Diagram](diagrams/findbyid_sequence.svg)

## <a name="provider">Provider</a>

### Overview

The initial Providers will implement basic resources - the AmbientAirTemperature property and the send_notification command.

### Interfaces

A Provider supports an interface for subscribing to resource's data feeds, requesting a resource's value, setting a resource's value and invoking a command.

### Activities

#### Subscribe

Below is the sequence diagram for the Subscribe activity.

![Sequence Diagram](diagrams/subscribe_sequence.svg)

#### Invoke

Below is the sequence diagram for the Invoke activity.

![Sequence Diagram](diagrams/invoke_sequence.svg)

## <a name="consumer">Consumer</a>

### Overview

The initial Consumers will provide the functionality needed by the proof-of-concept to subscribe to resources data feeds and invoke commands on resources.

Interfaces

A Consumer supports an interface that is the callback/notification endpoint for subscribed-to data feeds.

Activities

#### Publish

Below is the sequence diagram for the Publish activity.

![Sequence Diagram](diagrams/publish_sequence.svg)

#### Respond

Below is the sequence diagram for the Respond activity.

![Sequence Diagram](diagrams/respond_sequence.svg)

## <a name="appendix-a">Appendix A – Digital Twin Provider Interface</a>

### Subscribe

Subscribe to a property's data feed.

#### Request

- entity_id - The property's id.
- consumer_uri - The uri for the consumer endpoint where the data feed will be delivered.

#### Response

- No response.

### Unsubscribe

Unsubscribe from a property's data feed.

#### Request

- entity_id - The property's id.
- consumer_uri - The uri for the consumer endpoint where the data feed should no longer be delivered.

#### Response

- No response.

### Get

Get the latest value for a property and publish it to a consumer endpoint.

#### Request

- entity_id - The property's id.
- consumer_uri -  The uri for the consumer endpoint where the value should be delivered.

#### Response

- No response.

### Set

Set an entity's value to the one provided. This may not cause a change if the entity cannot be updated.

#### Request

- entity_id - The entity's id.
- value - The entity's new value.

#### Response

- No response.

### Invoke

Invoke a command.

#### Request

- entity_id - The command's id.
- consumer_uri - The uri for the endpoint where the command's response should be delivered.
- response_id - The id that the invoker of the command provided for the response.
- payload - The command's request payload.

#### Response

- No response.

## <a name="appendix-b">Appendix B – Digital Twin Interface</a>

### FindById

Find an entity's access information.

#### Request

- entity_id - The entity's id.

#### Response

- entity_access_info - The entity's access information.

### Register

Register one or more entities access information.

#### Request

- entity_access_info_list - A list of entity access information.

#### Response

- No response.

## <a name="appendix-c">Appendix C – Digital Twin Consumer Interface</a>

### Publish

Publish a value for a specific entity.

#### Request

- entity_id - The entity's id.
- value - The value to publish.

#### Response

- No response.

### Respond

Respond for the execution of a command.

#### Request

- entity_id - The command's id.
- response_id - The id that the invoker of the command provided for the response.
- payload - The command's response payload.

#### Response

- No response.
