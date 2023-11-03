# Samples Containerization

This document covers how to containerize the samples for Ibeji. To run the in-vehicle digital twin
service as a container or to get more in depth instruction for building and running a container
please refer to [Ibeji Containerization](../../container/README.md).

## Running the Samples in a Container

Below are the steps for running the sample providers and consumers in a container. Note that the
configuration files used by the containerized sample are cloned from
[/samples/container/config](./samples/container/config/) defined in the project's root.

### Provider

Provider containers utilize the dockerfile [Dockerfile.provider](../Dockerfile.provider).

#### Build

By default, the sample provider built in the Dockerfile is the
[property-provider](./samples/property/provider/). To change the provider the container builds, use
the following argument:

    <b>Docker</b>

    ```shell
    --build-arg APP_NAME={name of the provider}
    ```

    For example:

    ```shell
    docker build -t provider -f ./samples/Dockerfile.provider . --build-arg APP_NAME=managed-subscribe-provider
    ```

    <b>Podman</b>

    ```shell
    --build-arg=APP_NAME={name of the provider}
    ```

    For example:

    ```shell
    podman build -t provider:latest -f ./samples/Dockerfile.provider . --build-arg=APP_NAME=managed-subscribe-provider
    ```

#### Run

If you selected a different provider than the default provider built in the dockerfile, you may
need to override the configuration file [provider_settings.yaml](./config/provider/provider_settings.yaml).
To override the configuration of a provider container, follow the below steps after you have built
the container:

    <b>Docker</b>

    1. To run the container with overridden configuration, create your config file and set an
    environment variable called IBEJI_HOME to the path to the config file:

        ```shell
        export IBEJI_HOME={path to directory containing config file}
        ```

    1. Then run the container from the project's root with the following command:

        ```shell
        docker run -v ${IBEJI_HOME}:/sdv/config --name provider -p 5010:5010 --env-file=./samples/container/config/docker.env --add-host=host.docker.internal:host-gateway -it --rm invehicle_digital_twin
        ```

    <b>Podman</b>

    1. To run the container with overridden configuration, create your config file and set an
    environment variable called IBEJI_HOME to the path to the config file:

        ```shell
        export IBEJI_HOME={path to directory containing config file}
        ```

    1. Then run the container from the project's root with the following command:

        ```shell
        podman run --mount=type=bind,src=${IBEJI_HOME},dst=/sdv/config,ro=true -p 5010:5010 --env-file=./samples/container/config/podman.env --network=slirp4netns:allow_host_loopback=true localhost/provider:latest
        ```

### Consumer

Consumer containers utilize the dockerfile [Dockerfile.consumer](../Dockerfile.consumer).

#### Build

By default, the sample consumer built in the Dockerfile is the
[property-consumer](./samples/property/consumer/). To change the consumer the container builds, use
the following argument:

    <b>Docker</b>

    ```shell
    --build-arg APP_NAME={name of the consumer}
    ```

    For example:

    ```shell
    docker build -t consumer -f ./samples/Dockerfile.consumer . --build-arg APP_NAME=managed-subscribe-consumer
    ```

    <b>Podman</b>

    ```shell
    --build-arg=APP_NAME={name of the consumer}
    ```

    For example:

    ```shell
    podman build -t consumer:latest -f ./samples/Dockerfile.consumer . --build-arg=APP_NAME=managed-subscribe-consumer
    ```

#### Run

If you selected a different consumer than the default consumer built in the dockerfile, you may
need to override the configuration file [consumer_settings.yaml](./config/consumer/consumer_settings.yaml).
To override the configuration of a consumer container, follow the below steps after you have built
the container:

    <b>Docker</b>

    1. To run the container with overridden configuration, create your config file and set an
    environment variable called IBEJI_HOME to the path to the config file:

        ```shell
        export IBEJI_HOME={path to directory containing config file}
        ```

    1. Then run the container from the project's root with the following command:

        ```shell
        docker run -v ${IBEJI_HOME}:/sdv/config --name consumer -p 5010:5010 --env-file=./samples/container/config/docker.env --add-host=host.docker.internal:host-gateway -it --rm invehicle_digital_twin
        ```

    <b>Podman</b>

    1. To run the container with overridden configuration, create your config file and set an
    environment variable called IBEJI_HOME to the path to the config file:

        ```shell
        export IBEJI_HOME={path to directory containing config file}
        ```

    1. Then run the container from the project's root with the following command:

        ```shell
        podman run --mount=type=bind,src=${IBEJI_HOME},dst=/sdv/config,ro=true -p 5010:5010 --env-file=./samples/container/config/podman.env --network=slirp4netns:allow_host_loopback=true localhost/consumer:latest
        ```
