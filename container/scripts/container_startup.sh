#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# Exits immediately on failure.
set -e

# Copy any configuration files present to service configuration.
# If there is a configuration file with the same name at `/sdv/config` this will overwrite
# that file with the mounted configuration file.
cp -rf /mnt/config /sdv

# Start the Ibeji service.
/sdv/service