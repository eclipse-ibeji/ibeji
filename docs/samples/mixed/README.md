# Sample: Mixed

The mixed sample demonstrates the use of both properties and commands.

Follow these instructions to run the demo.

Steps:

1. The best way to run the demo is by using three windows: one running the In-Vehicle Digital Twin, one running the Digital Twin Provider and one running the Digital Twin Consumer.
Orientate the three windows so that they are lined up in a column. The top window can be used for the In-Vehicle Digital Twin.
The middle window can be used for the Digital Twin Provider. The bottom window can be used for the Digital Twin Consumer.<br>
1. In each window, change directory to the directory containing the build artifacts.
Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.<br><br>
`cd {repo-root-dir}/target/debug`<br>
1. Create the three config files with the following contents, if they are not already there:<br><br>
---- consumer_settings.yaml ----<br>
`consumer_authority: "0.0.0.0:6010"`<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>
---- invehicle_digital_twin_settings.yaml ----<br>
`invehicle_digital_twin_authority: "0.0.0.0:5010"`<br><br>
---- provider_settings.yaml ----<br>
`provider_authority: "0.0.0.0:4010"`<br>
`invehicle_digital_twin_uri: "http://0.0.0.0:5010"`<br><br>
1. In the top window, run:<br><br>
`./invehicle-digital-twin`<br>
1. In the middle window, run:<br><br>
`./mixed-provider`<br>
1. In the bottom window, run:<br><br>
`./mixed-consumer`<br>
1. Use control-c in each of the windows when you wish to stop the demo.

A templated version of each config file can be found in:

- {repo-root-dir}/core/invehicle-digital-twin/template
- {repo-root-dir}/samples/common/template
