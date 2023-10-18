#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

# Check if the correct number of arguments are provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 path_to_json_file path_to_text_files_directory"
    exit 1
fi

# Assign arguments to variables for clarity
json_file="$1"
text_files_dir="$2"

# Check if the JSON file exists
if [ ! -f "$json_file" ]; then
    echo "Error: JSON file '$json_file' not found"
    exit 1
fi

# Check if the text files directory exists
if [ ! -d "$text_files_dir" ]; then
    echo "Error: text files directory '$text_files_dir' not found"
    exit 1
fi

# Create a temporary file to store the updated JSON
temp_file=$(mktemp)

# Read JSON file and update elements with LicenseDescription field
while read -r line; do
    # Extract values from JSON object
    package_name=$(echo "$line" | jq -r '.PackageName')
    package_version=$(echo "$line" | jq -r '.PackageVersion')

    # Construct path to license description text file
    license_description_file="${text_files_dir}/${package_name}_${package_version}.txt"

    # Check if the license description text file exists
    if [ ! -f "$license_description_file" ]; then
        echo "Error: license description text file '$license_description_file' not found"
        exit 1
    fi

    # Read license description text file and add LicenseDescription field to JSON object
    license_description=$(cat "$license_description_file")
    updated_json=$(echo "$line" | jq --arg desc "$license_description" '. + {LicenseDescription: $desc}')

    # Write updated JSON object to temporary file
    echo "$updated_json" >> "$temp_file"
done < <(jq -c '.[]' "$json_file")

# Overwrite original JSON file with updated JSON from temporary file
jq -s '.' "$temp_file" > "$json_file"

# Remove temporary file
rm "$temp_file"

exit 0
