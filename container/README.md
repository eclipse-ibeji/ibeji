# Ibeji Containers

This document covers how to containerize the services provided in this repository.

## Dockerfile Selection

### In-Vehicle Digital Twin Service

To containerize the [In-Vehicle Digital Twin Service](../core/invehicle-digital-twin/) use one of
the following dockerfiles:

- [Dockerfile.amd64](../Dockerfile.amd64) - For x86-64 architecture.
- [Dockerfile.arm64](../Dockerfile.arm64) - For aarch64 architecture.

### Sample Applications

To containerize one of the [Ibeji Sample Applications](../samples/) use one of the following
dockerfiles:

- [./samples/Dockerfile.samples.amd64](../samples/Dockerfile.samples.amd64) - For x86-64
architecture.
- [./samples/Dockerfile.samples.arm64](../samples/Dockerfile.samples.arm64) - For aarch64
architecture.

## Configuration Defaults

### In-Vehicle Digital Twin Service

This repository provides default configuration files for the In-Vehicle Digital Twin Service
running in [Standalone](../container/config/standalone/) and
[Integrated](../container/config/integrated) modes.

>Note: Integrated mode runs the In-Vehicle Digital Twin Service with the
[Chariott Service](https://github.com/eclipse-chariott/chariott) and the
[Agemo Service](https://github.com/eclipse-chariott/Agemo). Other combinations may require config
changes.

### Sample Applications

For the samples, this repository provides
default configuration files for the [Property Provider](../samples/container/config/provider/) and
[Property Consumer](../samples/container/config/consumer/) samples.

>Note: Other samples may need config changes (see
[Running the Samples](../README.md#running-the-samples) for config changes).

## Docker Containers

### Prerequisites

[Install Docker](https://docs.docker.com/engine/install/)

### Running in Docker

To run the service in a Docker container:

>Note: Before running any of the following commands, replace all placeholders (wrapped with `<>`).

1. Run the following command in the project root directory to build the docker container from the
Dockerfile:

    ```shell
    docker build -t <image_name> -f <Dockerfile> [--build-arg=APP_NAME=<project name>] [--build-arg=FEATURES="<module name(s)>"] .
    ```

    >Note: The `FEATURES` build arg only applies to the dockerfiles for In-Vehicle Digital Twin Service.

    For example, to build an image for the `invehicle-digital-twin` project in `Standalone` mode:

    ```shell
    docker build -t invehicle_digital_twin -f Dockerfile.amd64 .
    ```

    Or to build an image for the `invehicle-digital-twin` project with the `Managed Subscribe`
    module:

    ```shell
    docker build -t invehicle_digital_twin -f Dockerfile.amd64 --build-arg=FEATURES="managed_subscribe" .
    ```

    >Note: Modules for Ibeji are enabled via the `--features` flag. The `FEATURES` build arg passes
    one or more features to the cargo build argument. If you want to add multiple features, add
    each feature followed by a space e.g. `"module_1 module_2"`.

    Or to build an image for the `property-provider` sample for aarch64:

    ```shell
    docker build -t property_provider -f ./samples/Dockerfile.samples.arm64 --build-arg=APP_NAME=property-provider .
    ```

    >Note: The build arg `APP_NAME` needs to be passed in for all sample applications to build the
    correct sample.

1. Once the container has been built, start the container in interactive mode with the following
command in the project root directory:

    ```shell
    docker run --name <container_name> --network=host -it --rm <image_name>
    ```

    >Note: Most images built will require configuration overrides. See
    [Running in Docker with overridden configuration](#running-in-docker-with-overridden-configuration)
    for more details.

    For example, to run the `invehicle_digital_twin` standalone image built in step 1:

    ```shell
    docker run --name invehicle_digital_twin --network=host -it --rm invehicle_digital_twin
    ```

    >Note: A custom network is recommended when using a container for anything but testing.

1. To detach from the container, enter:

    <kbd>Ctrl</kbd> + <kbd>p</kbd>, <kbd>Ctrl</kbd> + <kbd>q</kbd>

1. To stop the container, enter:

    ```shell
    docker stop <container_name>
    ```

    For example, to stop the `invehicle_digital_twin` container started in step 2:

    ```shell
    docker stop invehicle_digital_twin
    ```

### Running in Docker with overridden configuration

Follow the steps in [Running in Docker](#running-in-docker) to build the container.

1. To run the container with overridden configuration, create your config file and set an
environment variable called CONFIG_HOME to the path to the config file:

    ```shell
    export CONFIG_HOME={path to directory containing config file}
    ```

    For example, to set the configuration for the
    [Property Provider](../samples/property/provider/) sample, run:

    ```shell
    export CONFIG_HOME={path-to-repo-root}/samples/container/config/provider
    ```

    >Note: See [Configuration Defaults](#configuration-defaults) for more information. If running
    a sample other than the `Property` sample you will need to create your own configuration files.

1. Then run the container with the following command:

    ```shell
    docker run -v ${CONFIG_HOME}:/mnt/config --name <container_name> --network=host -it --rm <image_name>
    ```

    For example, to run the `property_provider` image with overridden configuration:

    ```shell
    docker run -v ${CONFIG_HOME}:/mnt/config --name property_provider --network=host -it --rm property_provider
    ```

## Podman

### Prerequisites

[Install Podman](https://podman.io/docs/installation)

### Running in Podman

To run the service in a Podman container:

>Note: Before running any of the following commands, replace all placeholders (wrapped with `<>`).

1. Run the following command in the project root directory to build the podman container from the
Dockerfile:

    ```shell
    podman build -t <image_name> -f <Dockerfile> [--build-arg=APP_NAME=<project name>] [--build-arg=FEATURES="<module name(s)>"] .
    ```

    >Note: The `FEATURES` build arg only applies to the dockerfiles for In-Vehicle Digital Twin Service.

    For example, to build an image for the `invehicle-digital-twin` project in `Standalone` mode:

    ```shell
    podman build -t invehicle_digital_twin -f Dockerfile.amd64 .
    ```

    Or to build an image for the `invehicle-digital-twin` project with the `Managed Subscribe`
    module:

    ```shell
    podman build -t invehicle_digital_twin -f Dockerfile.amd64 --build-arg=FEATURES="managed_subscribe" .
    ```

    >Note: Modules for Ibeji are enabled via the `--features` flag. The `FEATURES` build arg passes
    one or more features to the cargo build argument. If you want to add multiple features, add
    each feature followed by a space e.g. `"module_1 module_2"`.

    Or to build an image for the `property-provider` sample for aarch64:

    ```shell
    podman build -t property_provider -f ./samples/Dockerfile.samples.arm64 --build-arg=APP_NAME=property-provider .
    ```

    >Note: The build arg `APP_NAME` needs to be passed in for all sample applications to build the
    correct sample.

1. Once the container has been built, start the container with the following command in the project
root directory:

    ```shell
    podman run --network=host <image_name>
    ```

    >Note: Most images built will require configuration overrides. See
    [Running in Podman with overridden configuration](#running-in-podman-with-overridden-configuration)
    for more details.

    For example, to run the `invehicle_digital_twin` image built in step 1:

    ```shell
    podman run --network=host invehicle_digital_twin
    ```

    >Note: A custom network is recommended when using a container for anything but testing.

1. To stop the container, run:

    ```shell
    podman ps -f ancestor=<image_name> --format="{{.Names}}" | xargs podman stop
    ```

    For example, to stop the `invehicle_digital_twin` container started in step 2:

    ```shell
    podman ps -f ancestor=localhost/invehicle_digital_twin:latest --format="{{.Names}}" | xargs podman stop
    ```

### Running in Podman with overridden configuration

Follow the steps in [Running in Podman](#running-in-podman) to build the container.

1. To run the container with overridden configuration, create your config file and set an
environment variable called CONFIG_HOME to the path to the config file:

    ```shell
    export CONFIG_HOME={path to directory containing config file}
    ```

    For example, to set the configuration for the
    [Property Provider](../samples/property/provider/) sample, run:

    ```shell
    export CONFIG_HOME={path-to-repo-root}/samples/container/config/provider
    ```

    >Note: See [Configuration Defaults](#configuration-defaults) for more information. If running
    a sample other than the `Property` sample you will need to create your own configuration files.

1. Then run the container with the following command:

    ```shell
    podman run --mount=type=bind,src=${CONFIG_HOME},dst=/mnt/config,ro=true --network=host <image_name>
    ```

    For example, to run the `property_provider` image with overridden configuration:

    ```shell
    podman run --mount=type=bind,src=${CONFIG_HOME},dst=/mnt/config,ro=true --network=host property_provider
    ```
