#!/bin/sh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

sudo apt update
sudo apt install -y snapd

echo "Installing Rust"
sudo snap install rustup --classic
rustup toolchain install nightly-2022-08-11
echo "You will need to exit this terminal and start a new one to get all of the Rust tools in your path"

echo "Installing the Protobuf Compiler"
sudo apt install -y protobuf-compiler

echo "Successfully completed"