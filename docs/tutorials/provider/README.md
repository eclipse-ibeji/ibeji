# Tutorial: Create a Digital Twin Provider

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [1. Create an Ibeji Digital Twin Provider](#1-create-an-ibeji-digital-twin-provider)
  - [1.1 Define Digital Twin Provider Interface](#11-define-digital-twin-provider-interface)
    - [Sample Digital Twin Provider Interface](#sample-digital-twin-provider-interface)
  - [1.2 Implement the Operations of a Digital Twin Provider Interface](#12-implement-the-operations-of-a-digital-twin-provider-interface)
    - [Rust Sample Implementation of the Sample Interface](#rust-sample-implementation-of-the-sample-interface)
- [2. Register a Digital Twin Provider with the In-Vehicle Digital Twin Service](#2-register-digital-twin-provider-with-the-in-vehicle-digital-twin-service)
  - [2.1 Rust Sample Registration of a Digital Twin Provider](#21-rust-sample-registration-of-a-digital-twin-provider)
    - [Run the Sample Digital Twin Provider](#run-the-sample-digital-twin-provider)
- [3. Add Managed Subscribe to Digital Twin Provider](#3-add-managed-subscribe-to-digital-twin-provider)
- [Next Steps](#next-steps)

## Introduction

>[Digital Twin Provider:](../../../README.md#high-level-design) A provider exposes a subset of the vehicle's hardware capabilities by registering them with the In-Vehicle Digital Twin Service. Once registered with the In-Vehicle Digital Twin Service they can in turn be offered to Ibeji consumers. Each capability includes metadata that allows Ibeji consumers to comprehend the nature of the capability, how to work with it and how it can be remotely accessed.

In this tutorial, you will leverage your in-vehicle model in code that you have created in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md) to create a digital twin provider. Additionally, you will learn how to register your digital twin provider with the [In-Vehicle Digital Twin Service](../../design/README.md#in-vehicle-digital-twin-service).

This tutorial will reference the sample code provided in Ibeji to keep the tutorial concise. Relevant code snippets are explicitly highlighted and discussed to ensure a clear understanding of the concepts.

## Prerequisites

- Complete the tutorial in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md).
- Basic knowledge about [Protocol Buffers (protobufs) version 3.0](https://protobuf.dev/programming-guides/proto3/).
- Basic knowledge about the [gRPC protocol](https://grpc.io/docs/what-is-grpc/introduction/).

## 1. Create an Ibeji Digital Twin Provider

In this section, you will learn how to develop a digital twin provider that communicates with its digital twin consumers via [gRPC](https://grpc.io/docs/what-is-grpc/introduction/). It is important to note that digital twin providers in Ibeji are protocol-agnostic. This means they are not restricted to using gRPC and can employ other communication protocols.

The `{repo-root-dir}/samples/tutorial` directory contains code for the sample digital twin provider used in this tutorial. The `{repo-root-dir}/digital-twin-model/src` contains the in-vehicle model in Rust code that you have constructed in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md) along with additional signals not needed for this tutorial.

Throughout this tutorial, the sample contents in the `{repo-root-dir}/samples/tutorial` directory are referenced to guide you through the process of creating a digital twin provider.

### 1.1 Define Digital Twin Provider Interface

A digital twin provider needs an interface. The interface will expose operations that allow digital twin consumers to access a subset of in-vehicle signals that your digital provider makes available.

>Tip: A suggested approach for defining your digital twin provider is to adopt the perspective of a digital twin consumer. This requires consideration of the operations and their corresponding names for interacting with each in-vehicle signal and command. For example, for the [digital twin provider sample interface](../../../samples/interfaces/tutorial/digital_twin_provider.proto), the specified operations are `Get` and `Invoke`.

The [digital twin provider tutorial interface](../../../samples/interfaces/tutorial/digital_twin_provider.proto) serves as an example of what a digital twin provider's interface could look like. Feel free to replicate these operation names, modify them, or even add new ones as per your requirements. These operations are non-prescriptive. It is up to the developers of the in-vehicle digital twin to come up with their own convention for the operations.

#### Sample Digital Twin Provider Interface

This section provides an example of a digital twin provider interface. To reiterate, you are free to use this interface as a starting point or you may come up with your own convention.

1. Consider the in-vehicle signals *ambient air temperature* and *is air conditioning active*, as well as the command *show notification* that you defined in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md).

1. Reference the [sample digital twin provider tutorial interface](../../../samples/interfaces/tutorial/digital_twin_provider.proto):

In this digital twin provider sample interface, the conventions that this interface enforces are as follows:

- A digital twin consumer should utilize the `Get` operation to synchronously consume the *ambient air temperature* and the *is air conditioning active* in-vehicle signals.

- A digital twin consumer should utilize the `Invoke` operation to send a *show notification* command.

When introducing additional signals and commands, it is crucial to carefully select the operation(s) that best align with the behavior of each signal or command. This ensures a seamless integration and optimal performance of your system.

### 1.2 Implement the Operations of a Digital Twin Provider Interface

You have defined your digital twin provider interface.

The following lists out the flow for implementing the operations of a digital twin interface in the programming language of your choice:

1. Choose Your Programming Language: Since operations can be defined in a protobuf file, you can select any programming language that supports protobufs. This includes languages like Rust, Python, Java, C++, Go, etc. However, operations do not need to be defined in a protobuf file to be programming language agnostic. For instance, if you have a subscribe operation you may want to use [MQTT](https://mqtt.org/) for publishing to digital twin consumers that have subscribed to your digital twin provider. Please see the [Managed Subscribe Sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/managed_subscribe) and [Property Sample](../../../samples/property/provider/src/main.rs) for Rust examples of a digital twin provider using MQTT.

1. In your implementation, import the code of your in-vehicle digital twin model that you have created in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md#3-translating-dtdl-to-code).

1. Implement the operations you have defined in your interface. This involves writing the logic for what should happen to each in-vehicle signal or command when each operation is called. If you are using the [sample digital twin provider interface](#sample-digital-twin-provider-interface), you need to implement the functionality for the `Get` and `Invoke` operations.

1. For each opeartion you implement, you can reference an in-vehicle signal or command using the code of your in-vehicle digital twin model.

In order to translate your operations into code, it is important to understand the requirements of each operation.

#### Rust Sample Implementation of the Sample Interface

This section uses the [sample digital twin provider interface](#sample-digital-twin-provider-interface) that is defined in a protobuf file, and covers a *sample* Rust implementation of the synchronous `Get` and `Invoke` operations.

1. Reference the [code for implementing the operations for the sample digital twin provider interface](../../../samples/tutorial/provider/src/provider_impl.rs).

1. There is an import statement for the Rust in-vehicle digital twin model that you have previously constructed in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md#3-translating-dtdl-to-code):

```rust
use digital_twin_model::sdv_v1 as sdv;
```

1. The implementation of the `Get` operation references the signals *is air conditioning active* and *ambient air temperature*:

```rust
  /// Get implementation.
  ///
  /// # Arguments
  /// * `request` - Get request.
  async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {..}
```

1. The implementation of the `Invoke` operation references the command *show notification*.

```rust
/// Invoke implementation.
///
/// # Arguments
/// * `request` - Invoke request.
async fn invoke(
    &self,
    request: Request<InvokeRequest>,
) -> Result<Response<InvokeResponse>, Status> {..}
```

## 2. Register Digital Twin Provider with the In-Vehicle Digital Twin Service

You have defined your digital twin provider interface, and you have implemented the functionality of each operation in the programming language of your choice.

You will need to register your digital twin provider with the [In-Vehicle Digital Twin Service](../../../README.md#high-level-design). This registration will make your digital twin provider discoverable by digital twin consumers through the In-Vehicle Digital Twin Service.

>[In-Vehicle Digital Twin Service:](../../../README.md#high-level-design) Ibeji's architecture has an In-Vehicle Digital Twin Service at its core. The In-Vehicle Digital Twin Service captures all of the vehicle's primary capabilities and makes them available to Ibeji consumers.

The following lists out the flow for registering a digital twin provider in the programming language of your choice:

1. Reference the interface of the [In-Vehicle Digital Twin Service](../../../interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto) which is defined as a protobuf file.

1. In the code for your digital twin provider, you will need to import an `In-Vehicle Digital Twin Service` gRPC client.

1. Using the `In-Vehicle Digital Twin Service` gRPC client, you will need to define how to register your in-vehicle signals and commands with the In-Vehicle Digital Twin Service. This involves calling the `Register` gRPC method with the gRPC client.

### 2.1 Rust Sample Registration of a Digital Twin Provider

This section uses the [sample digital twin provider interface](#sample-digital-twin-provider-interface), and covers a *sample* Rust implementation of a provider registering the signals *ambient air temperature* and *is air conditioning active* and the command *show notification*

1. Reference the [main.rs file of the sample digital twin provider](../../../samples/tutorial/provider/src/main.rs). The main.rs file outlines the behavior of the signals in your digital twin provider sample. This includes a vehicle simulator that can emulate changes in its signals. These changes are then published to any digital twin consumers that have subscribed to your digital twin provider.

1. One function of particular interest in the [main.rs](../../../samples/tutorial/provider/src/main.rs) file is the `register_entities` function.

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

The `register_entities` function in this Rust sample digital twin provider showcases the process of registering with the In-Vehicle Digital Twin Service.

#### Run the Sample Digital Twin Provider

1. The best way to run the demo is by using three windows: one running the In-Vehicle Digital Twin, one running the Digital Twin Provider and one running the Digital Twin Consumer. Orientate the three windows so that they are lined up in a column. The top window can be used for the In-Vehicle Digital Twin. The middle window can be used for the Digital Twin Provider. The bottom window can be used for the Digital Twin Consumer.
In each window, change the current directory to the directory containing the build artifacts. Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.

1. cd {repo-root-dir}/target/debug
Create the three config files with the following contents, if they are not already there:

---- consumer_settings.yaml ----
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`

---- invehicle_digital_twin_settings.yaml ----
`invehicle_digital_twin_authority: "0.0.0.0:5010"`

---- provider_settings.yaml ----
`provider_authority: "0.0.0.0:4010"`
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`

1. In the top window, run:

`./invehicle-digital-twin`

1. In the middle window, run:

`./digital-twin-provider-tutorial`

1. In the bottom window, run:

`./digital-twin-consumer-tutorial`

1. Use control-c in each of the windows when you wish to stop the demo.

## 3. Add Managed Subscribe to Digital Twin Provider

>[Managed Subscribe:](../../../samples/managed_subscribe/README.md#introduction) The managed subscribe sample shows how Ibeji can extend its functionality with modules to give providers and consumers more capabilities. This sample utilizes the 'Managed Subscribe' module to allow a consumer to get an MQTT subscription for the AmbientAirTemperature value of a vehicle at a specific frequency in milliseconds. The provider, signaled with the help of the module, will publish the temperature value at the requested frequency for each consumer on its own topic and once the consumer unsubscribes and disconnects it will stop publishing to that dynamically generated topic.

The current implementation of the Managed Subscribe Module expects to utilize the [Agemo Service](https://github.com/eclipse-chariott/Agemo/blob/main/README.md). This service currently requires the use of an MQTT broker for communication between publishers and subscribers.

Adding the `Managed Subscribe` module for your digital twin provider is **optional**. However, here are some reasons why you might want to consider using the `Managed Subscribe` module for your digital twin provider:

- Efficient Data Management: Allows your digital twin provider to efficiently manage the data being sent to its digital twin consumers. Your digital twin provider only needs to publish data when there is a change, so it reduces unnecessary data transmission.

- Customized Frequency: Digital twin consumers can specify the frequency at which they want to receive updates. This allows for more tailored data delivery and can improve a digital twin consumer's experience.

- Automated Topic Cleanup: The feature automatically stops publishing to a topic once all the digital twin consumers have unsubscribed. This helps in resource optimization and ensures that data is not being sent to inactive digital twin consumers.

- Scalability: Managed Subscribe can handle a large number of digital twin consumers, making it a good choice for your digital twin provider that is expected to have many digital twin consumers subscribed to it.

- Enhanced Capabilities: The Managed Subscribe module extends the functionality of a digital twin provider.

If you decide to incorporate the `Managed Subscribe` module into your digital twin provider, please consult the [Managed Subscribe interface](../../../interfaces/module/managed_subscribe/v1/managed_subscribe.proto), and the [documentation for the Managed Subscribe sample](../../../samples/managed_subscribe/README.md) for guidance. You will need to implement the proto methods that are defined in the `Managed Subscribe` interface. Since the interface is defined in a protobuf file, the `Managed Subscribe` module is program language agnostic.

### 3.1 Rust Sample Implementation of a Managed Subscribe Digital Twin Provider

Please refer to the [sample Rust code for the Managed Subscribe Sample provider](../../../samples/managed_subscribe/provider/src/) to see an example of how to integrate the Managed Subscribe module into a digital twin provider.
This sample Rust code contains an *ambient air temperature* signal, and does not include the in-vehicle signal *is air conditioning active* and the command *show notification*.

## Next Steps
