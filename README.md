# Project Eclipse Ibeji

- [Introduction](#introduction)
- [High-level Design](#high-level-design)
- [Prerequisites](#prerequisites)
  - [Install gcc](#install-gcc)
  - [Install Rust](#install-rust)
  - [Install Protobuf Compiler](#install-protobuf-compiler)
  - [Install SDL2 library](#install-sdl2-library)
  - [Install dotnet-sdk library](#install-dotnet-sdk-library)
  - [Install MQTT Broker](#install-mqtt-broker)
- [Cloning the Repo](#cloning-the-repo)
- [Building](#building)
  - [Tokio Console Support](#tokio-console-support)
- [Running the Tests](#running-the-tests)
- [Running the Samples](#running-the-samples)
- [Using Chariott](#using-chariott)
- [Running in a Container](#running-in-a-container)
- [Trademarks](#trademarks)

## <a name="introduction">Introduction</a>

Eclipse Ibeji aims to provide the capability to express a digital representation of the vehicle state and its capabilities
through an extensible, open and dynamic architecture that provides access to the vehicle hardware, sensors and capabilities.

## <a name="high-level-design">High-level Design</a>

Ibeji's architecture has an In-Vehicle Digital Twin Service at its core. The In-Vehicle Digital Twin Service captures all of the vehicle's hardware capabilities
and makes them available to Ibeji consumers. Another component in Ibeji's architecture is the Provider. A vehicle may have one or more providers.
A provider exposes a subset of the vehicle's hardware capabilities by registering them with the In-Vehicle Digital Twin Service. Once registered with the
In-Vehicle Digital Twin Service they can in turn be offered to Ibeji consumers. Each capability includes metadata that allows Ibeji consumers to comprehend
the nature of the capability, how to work with it and how it can be remotely accessed.

## <a name="prerequisites">Prerequisites</a>

We recommend the use of Ubuntu 22.04 for running the In-Vehicle Digital Twin Service. Other
operating systems or versions of Ubuntu may encounter issues. We invite others to help us find and
resolve any compatibility issues.

### <a name="install-gcc">Install gcc</a>

Rust needs gcc's linker, so you will need to install it. To install gcc, do the following:

```shell
sudo apt install gcc
```

### <a name="install-rust">Install Rust</a>

At this point in time, you will need to use a nightly release of Rust. While it is not ideal to rely on a nightly release, we should be able to rely on the
stable release of Rust sometime in the not too distant future when some of the Rust crates that we use can all rely on it as well. To install Rust, do the following:

```shell
sudo apt update
sudo apt install -y snapd
sudo snap install rustup --classic
```

The toolchain version is managed by the `rust-toolchain.toml` file. If you do not have the correct toolchain installed on your system, it will automatically be installed when needed (for example, when running `cargo build`), so there is no need to install it manually.

### <a name="install-protobuf-compiler">Install Protobuf Compiler</a>

You will need to install the Protobuf Compiler. This can be done by executing:

```shell
sudo apt install -y protobuf-compiler
```

### <a name="install-sdl2-library">Install SDL2 library</a>

You will need to install the libsdl2-dev library. This is used by the streaming sample to view
images. Install the libsdl2-dev library by executing:

```shell
sudo apt install -y libsdl2-dev
```

### <a name="install-dotnet-sdk-library">Install dotnet-sdk library</a>

You will need to install dotnet-sdk for the dtdl-tools crate. This can be done by executing:

```shell
sudo apt install -y dotnet-sdk-8.0
```

### <a name="install-mqtt-broker">Install MQTT Broker</a>

If you plan to run any of the samples that use MQTT, then you will need to install a MQTT Broker, like [Mosquitto](https://github.com/eclipse/mosquitto).
Instructions for installing Mosquitto can be found [here](https://github.com/eclipse/mosquitto).

## <a name="cloning-the-repo">Cloning the Repo</a>

The repo has two submodules [opendigitaltwins-dtdl](https://github.com/Azure/opendigitaltwins-dtdl) and [iot-plugandplay-models](https://github.com/Azure/iot-plugandplay-models) that provide DTDL context files and DTDL samples file. To ensure that these are included, please use the following command when cloning Ibeji's github repo:

````shell
git clone --recurse-submodules https://github.com/eclipse-ibeji/ibeji`
````

## <a name="building">Building</a>

Once you have installed the prerequisites, go to your enlistment's root directory and run:

````shell
cargo build
````

This will build all of the foundation libraries and executables.

Ibeji also has add-on modules that rely on feature flags to include them in the build. For example, to build Ibeji with the Digital Twin Graph and the Digital Twin Registry modules run:

````shell
cargo build --features "digital_twin_graph,digital_twin_registry"
````

### <a name="tokio-console-support">Tokio Console Support</a>

Ibeji has support for using the [tokio console](https://github.com/tokio-rs/console) for advanced debugging. To enable this support, you need to build with the `tokio_console` feature enabled and with the `tokio_unstable` config flag for the rust compiler:

```shell
RUSTFLAGS="--cfg tokio_unstable" cargo build --features tokio_console
```

Read the [tokio console documentation](https://github.com/tokio-rs/console) to learn how to install tokio console and how to run it.

Note that the tokio console will intercept trace-level logs, so these will not be visible when debugging with the tokio console.

## <a name="running-the-tests">Running the Tests</a>

After successfully building Ibeji, you can run all of the unit tests. To do this go to the enlistment's root directory and run:

````shell
cargo test
````

Currently, we have no integration tests or end-to-end tests.

## <a name="running-the-samples">Running the Samples</a>

There are currently six samples:

- [Property Sample](docs/samples/property/README.md) - demonstrates the use of a property
- [Command Sample](docs/samples/command/README.md) - demonstrates the use of a command
- [Mixed Sample](docs/samples/mixed/README.md) - demonstrates the mixed use of properties and commands
- [Seat Massager Sample](docs/samples/seat_massager/README.md) - demonstrates the use of get/set for a seat massager
- [Streaming Sample](docs/samples/streaming/README.md) - demonstrates the use of streaming
- [Digital Twin Graph Sample](docs/samples/digital_twin_graph/README.md) - demonstrates the use of the Digital Twin Graph Service

The samples' configuration files will be loaded from the current working directory by default,
but an `IBEJI_HOME` environment variable can be used to change the base configuration directory to a different one:

```shell
IBEJI_HOME=/etc/ibeji ./invehicle-digital-twin
```

The above example tells `invehicle-digital-twin` to load configuration files from `/etc/ibeji` instead of using
the current working directory.

With the samples, Chariott may be used to discover the in-vehicle digital twin service. We will discuss how to enable this feature in the section on [Using Chariott](#using-chariott).

## <a name="using-chariott">Using Chariott</a>

If you want the digital twin consumers and digital twin providers for each demo to use Chariott to discover the URI for the In-Vehicle Digital Twin Service,
rather than having it statically provided in their respective config file, then do the following before starting each demo:

1. Clone a copy of Chariott from GitHub (`https://github.com/eclipse-chariott/chariott`).
1. Build Chariott
1. Run Chariott:<br><br>
`cargo run -p service_discovery`<br>
1. In each of the the config files, add the setting:<br><br>
`chariott_uri: "http://0.0.0.0:50000"`<br>
1. In the consumer's config file and the provider's config file, remove the setting for invehicle_digital_twin_uri, so that the chariott_uri will be used to find the In-vehicle Digital Twin URI.<br>

## <a name="running-in-a-container">Running in a Container</a>

Please refer to [Ibeji Containers](./container/README.md#ibeji-containers) for information on how
build and run the In-Vehicle Digital Twin Service or the Sample Applications in a container.

## <a name="trademarks">Trademarks</a>

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
