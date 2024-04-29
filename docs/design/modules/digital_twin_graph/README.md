# Design Specification for Digital Twin Graph Service

- [Introduction](#introduction)
- [Architecture](#architecture)
- [Sequences](#sequences)

## <a name="introduction">Introduction</a>

Ibeji today provides the foundations for constructing and interacting with the a digital twin on an edge device.  These abilities are primitive and do not necessarily provide a consumer with the best experience.  They can be used as building blocks to build facades that provide a consumer with a better experience.  This design specifies a graph-based facade, which will be named the Digital Twin Graph service.


## <a name="architecture">Architecture</a>

Ibeji's Application Server, which we will refer to as "Digital Twin App Server", has a modular architecture that allows new services to readily be added and existing services to readily be removed.  It also has build-time feature switches for controlling which service should be available at run-time.  Ibeji's initial service, the Invehicle Digital Twin service, was developed before the adoption of the modular architecture, but it will eventually be migrated across to it.

We will introduce a new service named "Digital Twin Graph" that will provide a facade for the Invehicle Digital Twin service and the providers.  Ideally, the consumer will not need to directly interact with provider
endpoints.  Instead, they will interact with a graph structure that represents the digital twin,

Ibeji's existing Invehicle Digital Twin service needs some adjustments to support the Digital Twin Graph service.  There is a future plan plan to rename it as the Digital Twin Registry service.
We will introduce a modified form of the service under the name "Digital Twin Registry" and for now keep the existing functionality intact under the original name "Invehicle Digital Twin".

![Component Diagram](diagrams/digital_twin_graph_component.svg)

## <a name="sequences">Sequences</a>

### Find

![Find Sequence Diagram Diagram](diagrams/find_sequence.svg)

### Get

![Get Sequence Diagram](diagrams/get_sequence.svg)

### Set

![Get Sequence Diagram](diagrams/set_sequence.svg)

### Invoke

![Invoke Sequence Diagram](diagrams/invoke_sequence.svg)
