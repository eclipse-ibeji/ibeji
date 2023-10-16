#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

cd "$(dirname "$0")/.."

# Check if the correct number of argments are passed
if [ "$#" -lt 3 ] ; then
    echo "Usage: $0 <PATH_TO_NOTICE_FILE> <CSHARP_PROJ_DIRECTORY> <PATH_TO_LICENSE_URL_TO_LICENSE_MAPPINGS>"
    exit 1
fi

# Assign notice_file_path and dotnet_directory to arguments
notice_file_path="$1"
dotnet_directory="$2"
license_url_to_license_mappings="$3"

# Check if the notice file exists
if [ ! -f "$notice_file_path" ]; then
    echo "Error: Notice file '$notice_file_path' not found"
    exit 1
fi

if ! dotnet tool list --global | grep -q 'dotnet-project-licenses'; then
    dotnet tool install --global dotnet-project-licenses
fi

dotnet_licenses_output_directory="$dotnet_directory/dotnet_licenses_output"
mkdir -p "$dotnet_licenses_output_directory"
echo "Getting the .NET Third Party licenses"

dotnet-project-licenses -i $dotnet_directory -o -f "$dotnet_licenses_output_directory" -u --json -e -c \
    --licenseurl-to-license-mappings "$license_url_to_license_mappings"

./tools/dotnet_get_licenses.sh "$dotnet_licenses_output_directory/licenses.json" "$dotnet_directory/dotnet_licenses_output"
./tools/dotnet_append_to_notice.sh "$notice_file_path" "$dotnet_licenses_output_directory/licenses.json"

rm -r "$dotnet_licenses_output_directory"

exit 0