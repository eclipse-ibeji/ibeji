# Ibeji Tutorials

- [Introduction](#introduction)
- [Run the Tutorial](#run-the-tutorial)

## Introduction

It is recommend to adhere to the following sequence for tutorial completion:

1. [Tutorial: Create an In-vehicle Model With DTDL](./in_vehicle_model/README.md)

1. [Tutorial: Create a Digital Twin Provider](./provider/README.md)

1. [Tutorial: Create a Digital Twin Conusmer](./consumer//README.md)

## Run the Tutorial

1. The best way to run the demo is by using three windows: one running the In-Vehicle Digital Twin, one running the Digital Twin Provider and one running the Digital Twin Consumer. Orientate the three windows so that they are lined up in a column. The top window can be used for the In-Vehicle Digital Twin. The middle window can be used for the Digital Twin Provider. The bottom window can be used for the Digital Twin Consumer.
In each window, change the current directory to the directory containing the build artifacts. Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.

1. cd {repo-root-dir}/target/debug
Create the three config files with the following contents, if they are not already there:

---- consumer_settings.yaml ----

```yaml
invehicle_digital_twin_uri: "http://0.0.0.0:5010"
```

---- invehicle_digital_twin_settings.yaml ----

```yaml
invehicle_digital_twin_authority: "0.0.0.0:5010"
```

---- provider_settings.yaml ----

```yaml
provider_authority: "0.0.0.0:4010"
invehicle_digital_twin_uri: "http://0.0.0.0:5010"
```

1. In the top window, run:

    `./invehicle-digital-twin`

1. In the middle window, run:

    `./digital-twin-provider-tutorial`

1. In the bottom window, run:

    `./digital-twin-consumer-tutorial`

1. Use control-c in each of the windows when you wish to stop the demo.
