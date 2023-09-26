## Managed Subscribe Sample

### Introduction

The managed subscribe sample shows how ibeji can extend its functionality with modules to give
providers and consumers more capabilities. This sample utilizes the 'Managed Subscribe' module to
allow a consumer to get an mqtt subscription for the AmbientAirTemperature value of a vehicle at a
specific frequency in milliseconds. The provider, through the module, will publish the temperature
value at the requested frequency for each consumer on its own topic and once the consumer
disconnects it will stop publishing to that dynamically generated topic.

### Setup

1. Create the four config files with the following contents, if they are not already there:<br><br>
---- consumer_settings.yaml ----<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>
---- invehicle_digital_twin_settings.yaml ----<br>
`invehicle_digital_twin_authority: "0.0.0.0:5010"`<br><br>
---- managed_subscribe_settings.yaml ----<br>
`base_authority: "0.0.0.0:5010"`<br>
`managed_subscribe_uri: "http://0.0.0.0:50051"`<br><br>
---- provider_settings.yaml ----<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>

1. Build the invehicle_digital_twin service with the `managed_subscribe` feature enabled.

### Running the Sample

#### Steps:

This sample uses [Agemo](https://github.com/eclipse-chariott/Agemo); please make sure that it is
running.

1. The best way to run the demo is by using three windows: one running the In-Vehicle Digital Twin,
one running the Digital Twin Provider and one running the Digital Twin Consumer.
Orientate the three windows so that they are lined up in a column. The top window can be used for
the In-Vehicle Digital Twin.
The middle window can be used for the Digital Twin Provider. The bottom window can be used for the
Digital Twin Consumer.<br>
1. In each window, change directory to the directory containing the build artifacts.
Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine
where you are running the demo.<br><br>
`cd {repo-root-dir}/target/debug`

1. In the top window, run:<br><br>
`./invehicle-digital-twin`

1. In the middle window, run:<br><br>
`./property-provider`

1. In the bottom window, run:<br><br>
`./property-consumer freq_ms=<value>`
    > Note: The consumer currently has a minimum frequency rate of 1000 ms, so set accordingly.
    Default is 10000 ms (10 secs) if no value is set in the optional `freq_ms` flag.<br>
    > Note: One or more consumers can be run with different frequencies. This will show how a
    provider can serve the same value in different ways dynamically. If one consumer is stopped,
    the other consumers will still receive the data.

1. To shutdown, use control-c on the consumer first. This will show the topic thread being shutdown
in the provider. Then control-c the other windows when you wish to stop the demo.

### Managed Subscribe Module

The managed subscribe module utilizes the [Agemo](https://github.com/eclipse-chariott/Agemo)
service to provide dynamic topic creation and subscription management. The module checks a
provider's registration request to see if the provider is requesting the use of the module. If so,
the module will inject the module's gRPC service endpoint for consumers to communicate with to
request a subscription for an entity id with specific constraints. In this example, the constraint
is frequency. Once a consumer has requested a subscription, the module will create a dynamic topic
through Agemo and tell the relevant provider to start publishing to that topic with the specific
constraints.

#### Register Interceptor Sequence

This diagram shows the portion of the module that handles the modification of a provider's
registration data.

![interceptor_sequence_diagram](../../docs/design/diagrams/managed_subscribe_interceptor_sequence.svg)

#### Managed Subscribe Module Sequence

This diagram shows the portion of the module that handles the managed topic creation and management
through Agemo for the providers.

![managed_subscribe_sequence_diagram](../../docs/design/diagrams/managed_subscribe_module_sequence.svg)
