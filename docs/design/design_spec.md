# Design Specification for Ibeji

- [Introduction](#introduction)
- [Architecture](#architecture)
- [DTDL](#dtdl)
- [In-Vehicle Digital Twin Service](#in-vehicle-digital-twin-service)
- [Provider](#provider)
- [Consumer](#consumer)

## <a name="introduction">Introduction</a>

This project delivers an In-Vehicle software component that is a digital representation of vehicle hardware resources.  The representation is usable by other software in the vehicle to read/write/query vehicle hardware resources in a standardized manner.

Please note that the initial Ibeji implementation is a proof-of-concept. We would like to see it evolve into an enterprise class solution.

## <a name="architecture">Architecture</a>

Ibeji has three main architectural concepts:

- Consumer
- Provider.
- In-Vehicle Digital Twin Service

The first Ibeji architectural concept that we will introduce is the Consumer. A Consumer is a software entity that utilizes Ibeji to interface with the digital representation of the In-Vehicle hardware components.

Another Ibeji architectural concept is the Provider. A Provider is the access point to some/all of the vehicle's hardware resources.  A Provider registers itself with the In-Vehicle Digital Twin Service. Once registered, the In-Vehicle Digital Twin Service can make the resources available to Consumers. Each resource includes meta data that allow Consumers to understand the semantics of the resource and know how to interact with it. The In-Vehicle Digital Twin Service supports multiple simultaneous Providers and internally resolves overlapping resources offered by multiple Providers. These overlaps offer multiple options for interacting with a resource and can improve the resource's availability (by supporting multiple access paths). A Provider must support a Provider interface that enables access to resource data feeds.

In the middle is the In-Vehicle Digital Twin Service. It exports a query interface that enables Consumers to discover the vehicle's resources and provides the details necessary to use those resources. The In-Vehicle Digital Twin Service has an interface that allows Providers to dynamically register and unregister resources.

Below is the component diagram for Ibeji.

![Component Diagram](diagrams/ibeji_component.svg)

## <a name="dtdl">DTDL</a>

Fundamental to the Ibeji solution is its use of Digital Twin Definition Language [DTDL](https://github.com/Azure/opendigitaltwins-dtdl) to identify and specify each of the vehicle's resources, and to provide the metadata needed to interact the resource.

This initial contribution does not try to arrange the resources into a hierarchy or into a graph. It is intended that some future update will enable this capability.

DTDL can identify and specify each of the resources. DTDL allows additional metadata to be associated with each of the resources, specifically the endpoint that can be used to interact with that resource. Below is an example for the AmbientAirTemperature property. You can see that the resource has the "RemotelyAccessible" type, which allows it to specify remote access metadata. The remote_access element utilizes an "Endpoint" type to specify the resource's endpoint and the supported operations.

```uml
  {
    "@context": ["dtmi:dtdl:context;2", "dtmi:sdv:context;3"],
    "@type": "Interface",
    "@id": "dtmi:org:eclipse:sdv:interface:cabin:AmbientAirTemperature;1",
    "contents": [
      {
        "@type": ["Property", "Temperature", "RemotelyAccessible"],
        "@id": "dtmi:org:eclipse:sdv:property:cabin:AmbientAirTemperature;1",
        "name": "Cabin_AmbientAirTemperature",
        "description": "The immediate surroundings air temperature (in Fahrenheit).",
        "schema": "double",
        "unit": "degreeFahrenheit",
        "remote_access": [
          {
            "@type": "Endpoint",
            "uri": "http://[::1]:40010",
            "operations": [ "Get", "Set", "Subscribe", "Unsubscribe" ]
          }
        ]
      }
    ]
  }
```

The DTDL must use the standard dtmi dtdl context. It must also use the dtmi sdv context, which provides the definitions for the RemotelyAccessible type and the remote_access element.

## <a name="in-vehicle-digital-twin-service">In-Vehicle Digital Twin Service</a>

### In-Vehicle Digital Twin Service Overview

The initial In-Vehicle Digital Twin Service will provide the functionality needed by the proof-of-concept. On the Provider side, this initial contribution supports only a single Provider registering its DTDL. On the Consumer side, there is a simplified query api and the ability to subscribe to a provided hardware resource data feed.

### Interfaces

The initial In-Vehicle Digital Twin Service supports both Providers and Consumers with a gRPC interface.

### Activities

#### Register

Below is the sequence diagram for the Register activity.

![Sequence Diagram](diagrams/register_sequence.svg)

#### Find by Id

Below is the sequence diagram for the Find-By-Id activity.

![Sequence Diagram](diagrams/findbyid_sequence.svg)

## <a name="provider">Provider</a>

### Overview

The initial Provider will provide the functionality needed by the proof-of-concept, implementing one resource - the AmbientAirTemperature property.

### Interfaces

A Provider supports a gRPC interface for subscribing to resource's data feeds, unsubscribing, request a resource's value and setting a resource's value.

### Activities

#### Subscribe

Below is the sequence diagram for the Subscribe activity.  The Provider's endpoint details are exported by the Provider as DTDL to the Digital Twin Service.

![Sequence Diagram](diagrams/subscribe_sequence.svg)

## <a name="consumer">Consumer</a>

### Overview

The initial Consumer will provide the functionality needed by the proof-of-concept. It will only query and subscribe to one resource - the AmbientAirTemperature property. It will use the resource's endpoint metadata to subscribe.

Interfaces

A Consumer supports a gRPC interface that is the callback/notification endpoint for subscribed-to data feeds.

Activities

#### Publish

Below is the sequence diagram for the Publish activity.

![Sequence Diagram](diagrams/publish_sequence.svg)

## <a name="appendix-a">Appendix A – Provider gRPC Interface</a>

### Subscribe

Subscribe to a resource's data feed and publish the resource's updates to a Consumer using the Publish operation on its Consumer interface.

#### Request

- id - The resource's id.
- uri - The uri for the endpoint where the data feed will be delivered.

#### Response

- No response.

### Unsubscribe

Unsubscribe from a resource's data feed.

#### Request

- id - The resource's id.
- uri - The uri for the endpoint where the data feed should no longer be delivered.

#### Response

- No response.

### Get

Get the latest value for a resource and publish it to a Consumer using the Publish operation on its Consumer interface.

#### Request

- id - The resource's id.
- uri -  The uri for the endpoint where the value should be delivered.

#### Response

- No response.

### Set

Set a resource's value to the one provided. This may not cause a change if the resource cannot be updated.

#### Request

- id - The resource's id.
- value - The resource's new value.

#### Response

- No response.

## <a name="appendix-b">Appendix B – Digital Twin gRPC Interface</a>

### FindById

Find a resource's DTDL.

#### Request

- id - The resource's id.

#### Response

- dtdl - The resource's DTDL.

### Register

Register one or more resources.

#### Request

- dtdl - The DTDL that represents the resource/s.

#### Response

- No response.

### Unregister

Unregister a resource.

#### Request

- id - The resource's id.

#### Response

- No response.

## <a name="appendix-c">Appendix C – Consumer gRPC Interface</a>

### Publish

Publish a resource value.

#### Request

- id - The resource's id.
- value - The resource's value.

#### Response

- No response.
