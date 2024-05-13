# Design Specification for Digital Twin Graph Service

- [Introduction](#introduction)
- [Architecture](#architecture)
- [Identifiers](#identifiers)
- [Provider Contract](#provider-contract)
- [Operations](#operations)

## <a name="introduction">Introduction</a>

The initial Ibeji implementation provided the foundations for constructing and interacting with a digital twin on an edge device. These foundations are low-level abilities
and they do not necessarily provide a consumer with the best interaction experience. However, they can be used as building blocks to build facades that
provide a consumer with an abstraction that delivers a much better interaction experience. Ibeji may support multiple facades and the user can select the one
that they prefer to use.

This design specifies a graph-based facade, which will be named the Digital Twin Graph Service. With this facade, the digital twin will be represented as a
graph of digital twin entities whose edges represent the relationships between those entities.  Instance IDs will be used to refer to entities.

Please note that Ibeji is only intended for use on an IoT edge device. It is not intended for use in the cloud. The data that it manages can be
transferred to the cloud, through components like [Eclipse Freyja](https://github.com/eclipse-ibeji/freyja).

## <a name="architecture">Architecture</a>

Ibeji's Application Server, which we will refer to as "Digital Twin App Server", has a modular architecture that allows services to readily be added and removed.
It also has build-time feature switches for controlling which service should be available at run-time. Ibeji's initial service, the In-vehicle Digital Twin
service, was developed before the adoption of the modular architecture, so it cannot be readily removed.

We will introduce a new service named "Digital Twin Graph" that will provide a facade for interactions with the In-vehicle Digital Twin Service and the
providers. Ideally, the consumer will not need to directly interact with provider endpoints. Instead, the consumer will interact with a graph structure that
represents the digital twin.

Ibeji's In-vehicle Digital Twin Service needs some adjustments to support the Digital Twin Graph Service. We will introduce a modified form of the service under the name "Digital Twin Registry" and keep the existing functionality intact, for now, under the original In-vehicle Digital Twin Service.

The Managed Subscriber Service is an optional service that provides integration with Agemo. The Managed Subscriber Service has been included in the component diagram for completeness' sake.

![Component Diagram](diagrams/digital_twin_graph_component.svg)

## <a name="identifiers">Identifiers</a>

The Digital Twin Graph will use a variety of identifiers. We will discuss the purpose of each.

The model ID is the identifier for a DTDL fragment. It is expressed as a [DTMI](https://github.com/Azure/opendigitaltwins-dtdl/blob/master/DTDL/v3/DTDL.v3.md#digital-twin-model-identifier).

A digital twin may be decomposed into digital twin entities. Each digital twin entity is defined by a fragment of the digital twin's model (specified in DTDL). The instance ID is the identifier for a digital twin entity. The instance ID must be universally unique.

The provider ID is the identifier for a Digital Twin Provider. The provider ID must be universally unique and it is up to the provider to ensure this.  The provider id may be associated with multiple instance IDs.

## <a name="provider-contract">Provider Contract</a>

The provider operations that will initially be supported by the digital twin graph are: Get, Set and Invoke.

Providers that want to participate in the digital twin graph, will need to do the following:
<ul>
  <li>Provide the async_rpc's Request interface with an ask operation that will use a targeted payload that has the following:
  <ul>
    <li>Get:
    <ul>
      <li>instance_id: set to the the target's instance ID</li?>
      <li>member_path: is optional; if it is empty, then it means the entire entity; if it is not empty, then it targets a specific member</li?>
      <li>operation: set to "Get"</li>
      <li>payload: is not required</li>
    </ul>
    </li>
    <li>Set:
    <ul>
      <li>instance_id: set to the the target's instance ID</li>
      <li>member_path: is optional; if it is empty, then it means the entire entity; if it is not empty, then it targets a specific member</li>
      <li>operation: set to "Set"</li>
      <li>payload: the value</li>
    </ul>
    <li>Invoke:
    <ul>
      <li>instance_id: set to the the target's instance ID</li>
      <li>member_path: the name of command to invoke</li>
      <li>operation: set to "Invoke"</li>
      <li>payload: the command's request payload</li>
    </ul>
    </li>
  </ul>
  <li>Return the result from a provider operation to the async_rpc's Response interface using with the answer operation that has a payload that has the following:
  <ul>
    <li>Get: The value of the target.
    </li>
    <li>Set: The payload is not required.
    </li>
    <li>Invoke: The command's response payload.
    </li>
  </li>
  </ul>
</ul>

## <a name="operations">Operations</a>

The Digital Twin Graph Service will support four operations:

- Find
- Get
- Invoke
- Set (this operation will be implemented in a later phase)

### Find

The Digital Twin Graph's find operation allows you to retrieve all instance values that have a specific model id.

![Find Sequence Diagram Diagram](diagrams/find_sequence.svg)

### Get

The Digital Twin's get operation allows you to retrieve an instance value. You can reduce the scope of the result by specifying a specific member path within the instance.

![Get Sequence Diagram](diagrams/get_sequence.svg)

### Set

The Digital Twin's set operation allows you to modify an instance value. You can reduce the scope of the change by specifying a specific member path within the instance.

![Get Sequence Diagram](diagrams/set_sequence.svg)

### Invoke

The Digital Twin's invoke operation allows you to call an instance's command. You can use the member path to specify which of the instance's command is to be performed.
![Invoke Sequence Diagram](diagrams/invoke_sequence.svg)
