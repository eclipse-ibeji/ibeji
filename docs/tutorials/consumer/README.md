# Tutorial: Create a Digital Twin Consumer

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [1. Create an Ibeji Digital Twin Consumer](#1-create-an-ibeji-digital-twin-consumer)
  - [1.1 Define the Interaction with a Digital Twin Provider](#11-define-the-interaction-with-a-digital-twin-provider)
    - [Rust Sample Implementation of the Interaction with a Digital Twin Provider](#rust-sample-implementation-of-the-interaction-with-a-digital-twin-provider)
- [2. Discover a Digital Twin Provider with the In-Vehicle Digital Twin Service](#2-discover-a-digital-twin-provider-with-the-in-vehicle-digital-twin-service)
  - [2.1 Rust Sample Discovery of a Digital Twin Provider](#21-rust-sample-discovery-of-a-digital-twin-provider)
- [3. Add Managed Subscribe to Digital Twin Consumer](#3-add-managed-subscribe-to-digital-twin-consumer)
  - [3.1 Rust Sample Implementation of a Managed Subscribe Digital Twin Consumer](#31-rust-sample-implementation-of-a-managed-subscribe-digital-twin-consumer)
- [Next Steps](#next-steps)

## Introduction

>[Digital Twin Consumer:](../../../docs/design/README.md#architecture) A Digital Twin Consumer is a software entity that utilizes Ibeji to interface with the digital representation of the In-Vehicle hardware components. In the [Tutorial: Create a Digital Twin Provider](../provider/README.md), you have learned that a `digital twin provider` exposes a subset of the in-vehicle's hardware capabilities. Each in-vehicle hardware capability includes metadata that allows digital twin consumers to comprehend the nature of the capability, how to work with it and how it can be remotely accessed.

In this tutorial, you will leverage your in-vehicle model in code that you have developed in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md), and the digital twin provider that you have created in the [Tutorial: Create a Digital Twin Provider](../provider/README.md). You will learn how to create a digital twin consumer. Using Ibeji's [In-Vehicle Digital Twin Service](../../design/README.md#in-vehicle-digital-twin-service), you will learn how to find the digital twin provider that exposes the relevant in-vehicle signals to the digital twin consumer.

This tutorial will reference the tutorial sample code in `{repo-root-dir}/samples/tutorial` to keep the tutorial concise. Relevant code snippets are explicitly highlighted and discussed to ensure a clear understanding of the concepts.

## Prerequisites

- Complete the [Tutorial: Create a Digital Twin Provider](../provider/README.md).
- Basic knowledge about [Protocol Buffers (protobufs) version 3.0](https://protobuf.dev/programming-guides/proto3/).
- Basic knowledge about the [gRPC protocol](https://grpc.io/docs/what-is-grpc/introduction/).

## 1. Create an Ibeji Digital Twin Consumer

In this section, you will learn how to develop a digital twin consumer that communicates with the [digital twin provider that you have created in the previous tutorial](../provider/README.md#rust-sample-implementation-of-the-sample-interface) via [gRPC](https://grpc.io/docs/what-is-grpc/introduction/). Remember digital twin providers in Ibeji are protocol-agnostic. This means they are not restricted to using gRPC and can employ other communication protocols.

To interact with your digital twin provider, the digital twin consumer must use the same communication protocol and understand the interface contract. The programming language of the digital twin consumer and the digital twin provider does not need to match.

The `{repo-root-dir}/samples/tutorial/consumer` directory contains code for the sample digital twin consumer used in this tutorial. The `{repo-root-dir}/digital-twin-model/src` contains the in-vehicle model in Rust code that you have constructed in [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md) along with additional signals not needed for this tutorial.

Throughout this tutorial, the sample contents in the `{repo-root-dir}/samples/tutorial` directory are referenced to guide you through the process of creating a digital twin consumer.

### 1.1 Define the Interaction with a Digital Twin Provider

You have defined your [digital twin provider's interface](../provider/README.md#11-define-digital-twin-provider-interface). A digital twin consumer needs to utilize that interface to communicate with your digital twin provider to access the in-vehicle signals that your digital provider makes available.

The following lists out the flow for your digital twin consumer to use the interface of a digital twin provider in the programming language of your choice:

1. Refer to your digital twin provider's interface, understand its contract and familiarize yourself with the communication protocol it uses.

1. Choose a programming language that supports both gRPC and your digital twin provider’s communication protocol. gRPC is required to communicate with the In-Vehicle Digital Twin Service. This will be described further in [2. Discover a Digital Twin Provider with the In-Vehicle Digital Twin Service](#2-discover-a-digital-twin-provider-with-the-in-vehicle-digital-twin-service). For instance, if your digital twin provider uses MQTT, then you should select a programming language that supports both MQTT and gRPC. This includes languages like Rust, Python, Java, C++, Go, etc.

1. In the implementation of your digital twin consumer, import the code of your in-vehicle digital twin model that you have created in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md#3-translating-dtdl-to-code), and call the operations of the digital twin provider as needed for the in-vehicle signals or commands. You can reference an in-vehicle signal or command using the code of your in-vehicle digital twin model.

#### Rust Sample Implementation of the Interaction with a Digital Twin Provider

This section uses the [sample digital twin provider interface](../provider/README.md#sample-digital-twin-provider-interface) that is defined in a protobuf file, and covers calling `Get` and `Invoke` operations that are defined in the [sample digital twin provider](../../../samples/tutorial/provider/src/provider_impl.rs). This is the same sample digital twin provider used in the [Tutorial: Create a Digital Twin Provider](../provider/README.md#rust-sample-implementation-of-the-sample-interface).

1. Reference the [code for the sample digital twin consumer](../../../samples/tutorial/consumer/src/main.rs).

1. There is an import statement for the Rust in-vehicle digital twin model that you have previously constructed in the [Tutorial: Create an In-Vehicle Model with DTDL](../in_vehicle_model/README.md#3-translating-dtdl-to-code):

    ```rust
    use digital_twin_model::{sdv_v1 as sdv, ...};
    ```

1. This sample digital twin consumer is aware of the sample digital twin provider's interface and that it uses gRPC as the communication protocol. Therefore, there is a gRPC client in its implementation.

    ```rust
    use samples_protobuf_data_access::tutorial_grpc::v1::digital_twin_provider_tutorial_client::DigitalTwinProviderTutorialClient;
    ```

1. The [sample digital twin provider](../../../samples/tutorial/provider/src/provider_impl.rs) has a `Get` operation defined. This operation is called in the sample digital consumer in the `send_get_request` method.

1. The [sample digital twin provider](../../../samples/tutorial/provider/src/provider_impl.rs) has an `Invoke` operation defined. This operation is called in the sample digital consumer in the `start_show_notification_repeater` method.

### 2. Discover a Digital Twin Provider with the In-Vehicle Digital Twin Service

You have defined your [digital twin provider's interface](../provider/README.md#11-define-digital-twin-provider-interface). You have defined the interactions that your digital twin consumer needs to communicate with your digital twin provider to access the in-vehicle signals that your digital provider makes available.

Your digital twin consumer will need to discover your digital twin provider using the [In-Vehicle Digital Twin Service](../../../README.md#high-level-design) before the digital twin consumer can interact with the provider.

The following lists out the flow for your digital twin consumer to discover a digital twin provider in the programming language of your choice:

1. Reference the interface of the [In-Vehicle Digital Twin Service](../../../interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto) which is defined as a protobuf file.

1. In the code for your digital twin consumer, you will need to import an `In-Vehicle Digital Twin Service` gRPC client.

1. For each in-vehicle signal or command required by your digital twin consumer, it should utilize an `In-Vehicle Digital Twin Service` gRPC client to identify the corresponding digital twin provider. This involves calling the `FindById` gRPC method with the gRPC client. Please see the sequence diagram for [Find By Id](../../design/README.md#find-by-id) for more details.

### 2.1 Rust Sample Discovery of a Digital Twin Provider

This section uses the same *sample* Rust implementation of a digital twin consumer in [Rust Sample Implementation of the Interaction with a Digital Twin Provider](#rust-sample-implementation-of-the-interaction-with-a-digital-twin-provider). This digital twin consumer uses an `In-Vehicle Digital Twin Service` gRPC client to discover the signals *ambient air temperature* and *is air conditioning active*, as well as the command *show notification*.

1. Reference the [code of the sample digital twin consumer](../../../samples/tutorial/consumer/src/main.rs).

1. One function of particular interest in the [code of the sample digital twin consumer](../../../samples/tutorial/consumer/src/main.rs) is the `discover_digital_twin_provider_using_ibeji` helper function in the [sample commons utils](../../../samples/common/src/utils.rs). This function is a helper function for discovering a digital twin provider through the In-Vehicle Digital Twin Service.

    ```rust
    /// Use Ibeji to discover the endpoint for a digital twin provider that satifies the requirements.
    ///
    /// # Arguments
    /// * `invehicle_digitial_twin_service_uri` - In-vehicle digital twin service URI.
    /// * `entity_id` - The matching entity id.
    /// * `protocol` - The required protocol.
    /// * `operations` - The required operations.
    pub async fn discover_digital_twin_provider_using_ibeji(
        invehicle_digitial_twin_service_uri: &str,
        entity_id: &str,
        protocol: &str,
        operations: &[String],
    ) -> Result<EndpointInfo, String> {..}
    ```

    The `discover_digital_twin_provider_using_ibeji` function is called in the main function to discover the provider's endpoints for the signals *ambient air temperature* and *is air conditioning active*, as well as the command *show notification*. This Rust sample digital twin consumer shows the process of discovering a digital twin provider with the In-Vehicle Digital Twin Service.

1. When trying to discover a digital twin provider for an in-vehicle signal or command, your digital twin consumer can refer to the in-vehicle signal or command [code of your in-vehicle digital twin model](../in_vehicle_model/README.md#3-translating-dtdl-to-code).

## 3. Add Managed Subscribe to Digital Twin Consumer

In the previous [Tutorial: Create a Digital Twin Provider](../provider/README.md), the [Managed Subscribe module was introduced for digital twin providers](../provider/README.md#3-add-managed-subscribe-to-digital-twin-provider).

Please consult the [Managed Subscribe interface](../../../interfaces/module/managed_subscribe/v1/managed_subscribe.proto), and the [documentation for the Managed Subscribe sample](../../../samples/managed_subscribe/README.md) for guidance on developing a digital twin consumer to communicate with a digital twin provider that supports the `Managed Subscribe` module.

### 3.1 Rust Sample Implementation of a Managed Subscribe Digital Twin Consumer

Please refer to the [sample Rust code for the Managed Subscribe Sample Consumer](../../../samples/managed_subscribe/consumer/src/) to see an example of how to integrate the Managed Subscribe module into a digital twin consumer.
This sample Rust code contains an *ambient air temperature* signal, and does not include the in-vehicle signal *is air conditioning active* and the command *show notification*.

## Next Steps

- Run the tutorial by the following the steps in [Run the Tutorial](../README.md#run-the-tutorial)
