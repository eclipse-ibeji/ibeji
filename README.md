# Project Ibeji

- [Introduction](#introduction)
- [High-level Design](#high-level-design)
- [Prerequisites](#prerequisites)
  - [Install gcc](#install-gcc)
  - [Install Rust](#install-rust)
  - [Install Protobuf Compiler](#install-protobuf-compiler)
- [Cloning the Repo](#cloning-the-repo)
- [Developer Notes](#developer-notes)
  - [JSON-LD Crate](#json-ld-crate)
  - [DTDL Parser](#dtdl-parser)
- [Building](#building)
- [Running the Tests](#running-the-tests)
- [Running the Demo](#running-the-demo)
- [Trademarks](#trademarks)

## <a name="introduction">Introduction</a>

Eclipse Ibeji aims to provide the capability to express a digital representation of the vehicle state and its capabilities
through an extensible, open and dynamic architecture that provides access to the vehicle hardware, sensors and capabilities.

## <a name="high-level-design">High-level Design</a>

Ibeji's architecture has a In-Vehicle Digital Twin Service at its core. The In-Vehicle Digital Twin Service captures all of the vehicle's primary capabilities
and make them available to Ibeji consumers. Another component in Ibeji's architecture is the Provider. A vehicle may have one or more providers.
A provider exposes a subset of the vehicle's primary capabilities by registering them with the In-Vehicle Digital Twin Service. Once registered with the
In-Vehicle Digital Twin Service they can in turn be offered to Ibeji consumers. Each capability includes meta data that allow Ibeji consumers to comprehend
the nature of the capability, how to work with it and how it can be remotely accessed.

## <a name="prerequisites">Prerequisites</a>

### <a name="install-gcc">Install gcc</a>

Rust needs gcc's linker, so you will need to intsall it.  To install gcc, do the following:

```
sudo apt install gcc
```

### <a name="install-rust">Install Rust</a>

At this point in time, you will need to use the nightly release of Rust. While it is not ideal to rely on a nightly release, we should be able to rely on the
stable release of Rust sometime in the not too distant future when some of the Rust crates that we use can all rely on it as well. To install Rust, do the following:

```shell
sudo apt update
sudo apt install -y snapd
sudo snap install rustup --classic
rustup toolchain install nightly
rustup default nightly
```

If you have already installed Rust, but you are using another release, then you can switch to the nightly release by running the following commands:

```shell
rustup toolchain install nightly
rustup default nightly
```

### <a name="install-protobuf-compiler">Install Protobuf Compiler</a>

You will need to install the Protobuf Compiler. This can be done by executing:

`sudo apt install -y protobuf-compiler`

## <a name="cloning-the-repo">Cloning the Repo</a>

The repo has two submodule [opendigitaltwins-dtdl](https://github.com/Azure/opendigitaltwins-dtdl) and [iot-plugandplay-models](https://github.com/Azure/iot-plugandplay-models) that provide DTDL context files
and DTDL samples file.  To ensure that these are included, please use the following command when cloning Ibeji's github repo:

`git clone --recurse-submodules https://github.com/eclipse-ibeji/ibeji`

## <a name="developer-notes">Developer Notes</a>

### <a name="json-ld-crate">JSON-LD Crate</a>

Ideally, we should be using the json_ld 0.6.1 crate, which takes its source from [here](https://github.com/timothee-haudebourg/json-ld).
However, it currently has a build issue that is discussed [here](https://github.com/timothee-haudebourg/json-ld/issues/40).
To work around this issue you will need to use git clone to obtain the source from [here](https://github.com/blast-hardcheese/json-ld)
and checkout its "resolve-issue-40" branch. It should be cloned to a directory that is a sibling to ibeji.

### <a name="dtdl-parser">DTDL Parser</a>

There is no existing DTDL Parser for Rust, so we have provided a minimalistic one that is based on the [JavaScript DTDL Parser](https://github.com/Azure/azure-sdk-for-js/tree/%40azure/dtdl-parser_1.0.0-beta.2/sdk/digitaltwins/dtdl-parser).

## <a name="building">Building</a>

Once you have installed the prerequisites, go to your enlistment's root directory and run:

`cargo build`

This should build all of the libraries and executables.

## <a name="running-the-tests">Running the Tests</a>

After successfully building Ibeji, you can run all of the unit tests. To do this go to the enlistment's root directory and run:

`cargo test`

Currently, we have no integration tests or end-to-end tests.

## <a name="running-the-demo">Running the Demo</a>

Steps:

1. The best way to run the demo is by using three windows: one running the In-Vehicle Digital Twin, one running the Provider and one running a Consumer.
Orientate the three windows so that they are lined up in a column. The top window can be used for the In-Vehicle Digital Twin.
The middle window can be used for the Provider. The bottom window can be used for a Consumer.<br>
1. In each window run the following command too set the DTDL_PATH environment variable.
Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.<br><br>
`export DTDL_PATH="{repo-root-dir}/ibeji/opendigitaltwins-dtdl/DTDL;{repo-root-dir}/ibeji/dtdl;{repo-root-dir}/ibeji/samples/property/dtdl"`<br>
1. In each window change directory to the directory containing the build artifacts.
Make sure that you replace "{repo-root-dir}" with the repository root directory on the machine where you are running the demo.<br><br>
`cd {repo-root-dir}/ibeji/target/debug`<br>
1. In the top window, run:<br><br>
`./in-vehicle-digital-twin`<br>
1. In the middle window, run:<br><br>
`./property-provider`<br>
1. In the bottom window, run:<br><br>
`./property-consumer`<br>
1. Use control-c in each of the windows when you wish to stop the demo.

## <a name="trademarks">Trademarks</a>

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
