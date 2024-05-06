# Sample: Digital Twin Graph

The Digital Twin Graph sample demonstrates the use of the Digital Twin Graph Service.

This sample has two providers. The vehicle-core provider handles the vehicle, the vehicle's cabin and the cabin's seats.
The seat-massager provider handles all of the seat's seat masagers.

Follow these instructions to run the demo.

Steps:

1. The best way to run the demo is by using four windows: one running the In-Vehicle Digital Twin, two running the Digital Twin Providers and one running the Digital Twin Consumer.
Orientate the four windows so that they are lined up in a column. The top window can be used for the In-Vehicle Digital Twin.
The middle two windows can be used for the Digital Twin Providers. The bottom window can be used for the Digital Twin Consumer.<br>
1. In each window, change directory to the directory containing the build artifacts.
Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.<br>
`cd {repo-root-dir}/target/debug`<br><br>
1. Create the four config files with the following contents, if they are not already there:<br>
---- invehicle_digital_twin_settings.yaml ----<br>
`invehicle_digital_twin_authority: "0.0.0.0:5010"`<br><br>
---- digital_twin_graph_settings.yaml ----<br>
`base_authority: "0.0.0.0:5010"`<br><br>
---- consumer_settings.yaml ----<br>
`consumer_authority: "0.0.0.0:6010"`<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>
---- seat_massager_provider_settings.yaml ----<br>
`provider_authority: "0.0.0.0:4020"`<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>
1. In the top window, run:<br>
`./invehicle-digital-twin`<br><br>
1. In the second window, run:<br>
`./graph-vehicle-core-provider`<br><br>
1. In the third window, run:<br>
`./graph-seat-massager-provider`<br><br>
1. In the bottom window, run:<br>
`./graph-consumer`<br><br>
1. Use control-c in each of the windows when you wish to stop the demo.

A templated version of each config file can be found in:

- {repo-root-dir}/core/invehicle-digital-twin/template
- {repo-root-dir}/samples/common/template
