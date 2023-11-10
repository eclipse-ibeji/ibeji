# Tutorial: Create a Digital Twin Provider

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [1. Create an Ibeji Digital Twin Provider](#1-create-an-ibeji-digital-twin-provider)
  - [1.1 Define Digital Twin Provider Interface](#11-define-digital-twin-provider-interface)
    - [Sample Digital Twin Provider Interface](#sample-digital-twin-provider-interface)
  - [1.2 Implement the Operations for the Digital Twin Provider Interface](#12-implement-the-operations-for-the-digital-twin-provider-interface)
    - [Implement the Operations for the Sample Digital Twin Provider Interface](#implement-the-operations-for-the-sample-digital-twin-provider-interface)
- [2. Register Digital Twin Provider with the In-Vehicle Digital Twin Service](#2-register-digital-twin-provider-with-the-in-vehicle-digital-twin-service)
  - [2.1 Run Sample Digital Twin Provider](#21-run-sample-digital-twin-provider)
- [3. (Optional) Add Managed Subscribe to Digital Twin Provider](#3-optional-add-managed-subscribe-to-digital-twin-provider)
- [Next Steps](#next-steps)

## Introduction

>[Digital Twin Provider:](../../../README.md#high-level-design) "A provider exposes a subset of the vehicle's primary capabilities by registering them with the In-Vehicle Digital Twin Service. Once registered with the In-Vehicle Digital Twin Service they can in turn be offered to Ibeji consumers. Each capability includes metadata that allows Ibeji consumers to comprehend the nature of the capability, how to work with it and how it can be remotely accessed".

In this tutorial, you will leverage your in-vehicle model in code that you have created in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md) to create a digital twin provider. Additionally, you will learn how to register your digital twin provider with the [In-Vehicle Digital Twin Service](../../design/README.md#in-vehicle-digital-twin-service).

This tutorial will reference the sample code provided in Ibeji to keep the tutorial concise. Relevant code snippets are explicitly highlighted and discussed to ensure a clear understanding of the concepts.

## Prerequisites

- Complete the tutorial in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md).
- Basic knowledge about [Protocol Buffers version 3.0](https://protobuf.dev/programming-guides/proto3/).
- Basic knowledge about the [gRPC protocol](https://grpc.io/docs/what-is-grpc/introduction/).

## 1. Create an Ibeji Digital Twin Provider

In this section, you will learn how to develop a digital twin provider that communicates with its digital twin consumers via [gRPC](https://grpc.io/docs/what-is-grpc/introduction/). It is important to note that digital twin providers in Ibeji are protocol-agnostic. This means they are not restricted to using gRPC and can employ other communication protocols.

The `{repo-root-dir}/samples/mixed` directory contains code for the sample digital twin provider used in this tutorial. The `{repo-root-dir}/digital-twin-model/src` contains the in-vehicle model in Rust code that you have constructed in [Tutorial: Create an In-vehicle Model With DTDL](../in_vehicle_model/README.md) along with additional signals not needed for this tutorial.

Throughout this tutorial, the sample contents in the `{repo-root-dir}/samples/mixed` directory are referreced to guide you through the process of creating a digital twin provider.

### 1.1 Define Digital Twin Provider Interface

A digital twin provider needs an interface. The interface will expose operations that allow digital twin consumers to access a subset of in-vehicle signals that your digital provider makes available.

>Tip: A suggested approach to defining your digital twin provider is to adopt the perspective of a digital twin consumer. This involves consideration of the operations and their corresponding names. For example, for the [digital twin provider sample interface](../../../samples/interfaces/sample_grpc/v1/digital_twin_provider.proto), the specified operations are `Subscribe`, `Unsubscribe`, `Get`, `Set`, `Invoke` and `Stream`.

In this section, you will utilize the [digital twin provider sample interface](../../../samples/interfaces/sample_grpc/v1/digital_twin_provider.proto). Specifically, you will use the `Subscribe`, `Set` and `Invoke` operations from this interface.

>Please note that this interface serves as an example of what a digital twin provider's interface could look like. Feel free to replicate these operation names, modify them, or even add new ones as per your requirements.

#### Sample Digital Twin Provider Interface

1. Consider the in-vehicle signals *ambient air temperature* and *is air conditioning active*, as well as the command *show notification* that you defined in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md).

1. Reference the [digital twin provider sample interface](../../../samples/interfaces/sample_grpc/v1/digital_twin_provider.proto). In this tutorial, a digital twin consumer will only need to use the `Subscribe`, `Set` and `Invoke` operations. The digital twin consumer is covered in the next tutorial.

1. A digital twin consumer should utilize the `Subscribe` operation to consume the *ambient air temperature* and the *is air conditioning active* in-vehicle signals.

1. A digital twin consumer should utilize the `Set` operation to set the value of an in-vehicle signal.

1. A digital twin consumer should utilize the `Invoke` operation to send a *show notification* command.

### 1.2 Implement the Operations for the Digital Twin Provider Interface

You have defined your [digital twin provider interface](../../../samples/interfaces/sample_grpc/v1/digital_twin_provider.proto). Let's implement the functionality for the `Subscribe`, `Set` and `Invoke` operations.

#### Implement the Operations for the Sample Digital Twin Provider Interface

1. Reference the [code for implementing the operations for the sample digital twin provider interface](../../../samples/mixed/provider/src/provider_impl.rs). Please only consider the implementations for the `Subscribe`, `Set` and `Invoke` operations.

1. There is an import statement for the in-vehicle digital model that you have previously constructed in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md):

```rust
use digital_twin_model::sdv_v1 as sdv;
```

1. The implementation of the `Set` operation references the signals *is air conditioning active* and *ambient air temperature*.

1. The implementation of the `Invoke` operation references the command *show notification*.

## 2. Register Digital Twin Provider with the In-Vehicle Digital Twin Service

You have defined a sample interface with the following operations: `Subscribe`, `Set` and `Invoke` operations. You have implemented the functionality for each operation.

You will need to register your digital twin provider with the [In-Vehicle Digital Twin Service](../../../README.md#high-level-design). This registration will make your digital twin provider discoverable to digital twin consumers through the In-Vehicle Digital Twin Service.

>[In-Vehicle Digital Twin Service:](../../../README.md#high-level-design) "Ibeji's architecture has an In-Vehicle Digital Twin Service at its core. The In-Vehicle Digital Twin Service captures all of the vehicle's primary capabilities and makes them available to Ibeji consumers".

1. Reference the [main.rs file of your digital twin provider](../../../samples/mixed/provider/src/main.rs). The main.rs file outlines the behavior of the signals in your digital twin provider sample. This includes a vehicle simulator that can emulate changes in its signals. These changes are then published to any digital twin consumers that have subscribed to your digital twin provider.

1. One function of particular interest in the [main.rs](../../../samples/mixed/provider/src/main.rs) file is the register function.

```rust
/// Register the entities endpoints.
///
/// # Arguments
/// * `invehicle_digital_twin_uri` - The In-Vehicle Digital Twin URI.
/// * `provider_uri` - The provider's URI.
async fn register_entities(
    invehicle_digital_twin_uri: &str,
    provider_uri: &str,
) -> Result<(), Status> { .. }
```

The `register_entities` function in this sample digital twin provider exemplifies the process of registering with the In-Vehicle Digital Twin Service.

### 2.1 Run Sample Digital Twin Provider

Please refer to these [instructions](../../../README.md#mixed-sample) to run your sample digital twin provider.

## 3. (Optional) Add Managed Subscribe to Digital Twin Provider

>[Managed Subscribe:](../../../samples/managed_subscribe/README.md#introduction) "The managed subscribe sample shows how Ibeji can extend its functionality with modules to give providers and consumers more capabilities. This sample utilizes the 'Managed Subscribe' module to allow a consumer to get an MQTT subscription for the AmbientAirTemperature value of a vehicle at a specific frequency in milliseconds. The provider, through the module, will publish the temperature value at the requested frequency for each consumer on its own topic and once the consumer disconnects it will stop publishing to that dynamically generated topic".

Adding the `Managed Subscribe` module for your digital twin provider is optional. However, here are some reasons why you might want to consider using the `Managed Subscribe` module for your digital twin provider:

- Efficient Data Management: Allows your digital twin provider to efficiently manage the data being sent to its digital twin consumers. Your digital twin provider only needs to publish data when there is a change, so it reduces unnecessary data transmission.

- Customized Frequency: Digital twin consumers can specify the frequency at which they want to receive updates. This allows for more tailored data delivery and can improve a digital twin consumer's experience.

- Automated Unsubscription: The feature automatically stops publishing to a topic once all the digital twin consumers disconnect. This helps in resource optimization and ensures that data is not being sent to inactive digital twin consumers.

- Scalability: Managed Subscribe can handle a large number of digital twin consumers, making it a good choice for your digital twin provider that is expected to have many digital twin consumers subscribed to it.

- Enhanced Capabilities: The Managed Subscribe module extends the functionality of a digital twin provider.

If you decide to incorporate the `Managed Subscribe` module into your digital twin provider, please consult the [documentation for the Managed Subscribe Sample](../../../samples/managed_subscribe/README.md), and the [code for the Managed Subscribe Sample provider](../../../samples/managed_subscribe/provider/src/) for guidance.

## Next Steps
