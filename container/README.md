# Ibeji Containerization

This document covers how to containerize the In-Vehicle Digital Twin Service. To run the samples as
a container please refer to [Samples Containerization](../samples/container/README.md).

## Running the In-Vehicle Digital Twin Service in a Container

Below are the steps for running the service in a container. Note that the configuration files used
by the containerized service are cloned from [/container/config](./config/) defined in the
project's root.

### Dockerfile

There are currently two dockerfiles provided in the root directory of the project that can be built:

- Dockerfile - A standalone version of the In-Vehicle Digital Twin Service
- Dockerfile.integrated - A version of the In-Vehicle Digital Twin Service that communicates with
the [Chariott Service](https://github.com/eclipse-chariott/chariott) and the
[Agemo Service](https://github.com/eclipse-chariott/Agemo).

### Docker

#### Prerequisites

[Install Docker](https://docs.docker.com/engine/install/)

#### Running in Docker

To run the service in a Docker container:

1. Run the following command in the project's root directory to build the docker container from the
Dockerfile:

    ```shell
    docker build -t invehicle_digital_twin -f Dockerfile .
    ```

1. Once the container has been built, start the container in interactive mode with the following
command in the project's root directory:

    ```shell
    docker run --name invehicle_digital_twin -p 5010:5010 --env-file=./container/config/docker.env --add-host=host.docker.internal:host-gateway -it --rm invehicle_digital_twin
    ```

1. To detach from the container, enter:

    <kbd>Ctrl</kbd> + <kbd>p</kbd>, <kbd>Ctrl</kbd> + <kbd>q</kbd>

1. To stop the container, enter:

    ```shell
    docker stop invehicle_digital_twin
    ```

#### Running in Docker with overridden configuration

Follow the steps in [Running in Docker](#running-in-docker) to build the container.

1. To run the container with overridden configuration, create your config file and set an
environment variable called IBEJI_HOME to the absolute path of the directory containing the
config file:

    ```shell
    export IBEJI_HOME={absolute path of the directory containing the config file}
    ```

1. Then run the container with the following command:

    ```shell
    docker run -v ${IBEJI_HOME}:/sdv/config --name invehicle_digital_twin -p 5010:5010 --env-file=./container/config/docker.env --add-host=host.docker.internal:host-gateway -it --rm invehicle_digital_twin
    ```

### Podman

#### Prerequisites

[Install Podman](https://podman.io/docs/installation)

#### Running in Podman

To run the service in a Podman container:

1. Run the following command in the project's root directory to build the podman container from the
Dockerfile:

    ```shell
    podman build -t invehicle_digital_twin:latest -f Dockerfile .
    ```

1. Once the container has been built, start the container with the following command in the
project's root directory:

    ```shell
    podman run -p 5010:5010 --env-file=./container/config/podman.env --network=slirp4netns:allow_host_loopback=true localhost/invehicle_digital_twin
    ```

1. To stop the container, run:

    ```shell
    podman ps -f ancestor=localhost/invehicle_digital_twin:latest --format="{{.Names}}" | xargs podman stop
    ```

#### Running in Podman with overridden configuration

Follow the steps in [Running in Podman](#running-in-podman) to build the container.

1. To run the container with overridden configuration, create your config file and set an
environment variable called IBEJI_HOME to the absolute path of the directory containing the
config file:

    ```shell
    export IBEJI_HOME={absolute path of the directory containing the config file}
    ```

1. Then run the container with the following command:

    ```shell
    podman run --mount=type=bind,src=${IBEJI_HOME},dst=/sdv/config,ro=true -p 5010:5010 --env-file=./container/config/podman.env --network=slirp4netns:allow_host_loopback=true localhost/invehicle_digital_twin:latest
    ```
