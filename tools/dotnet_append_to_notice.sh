#!/bin/bash

set -e

# Check if the correct number of arguments are provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 path_to_markdown_file path_to_json_file"
    exit 1
fi

# Assign arguments to variables for clarity
markdown_file="$1"
json_file="$2"

# Check if the markdown file exists
if [ ! -f "$markdown_file" ]; then
    echo "Error: markdown file '$markdown_file' not found"
    exit 1
fi

# Check if the JSON file exists
if [ ! -f "$json_file" ]; then
    echo "Error: JSON file '$json_file' not found"
    exit 1
fi

# Append header to markdown file
echo -e "\n\n# .NET Third Party Licenses\nThe following lists the licenses of the .NET projects used.\n" >> "$markdown_file"

# Read JSON file and append information to markdown file
while read -r line; do
    # Extract values from JSON object
    license_type=$(echo "$line" | jq -r '.LicenseType')
    package_name=$(echo "$line" | jq -r '.PackageName')
    package_version=$(echo "$line" | jq -r '.PackageVersion')
    package_url=$(echo "$line" | jq -r '.PackageUrl')
    license_description=$(echo "$line" | jq -r '.LicenseDescription')

    # Append information to markdown file in specified format
    echo "### $license_type" >> "$markdown_file"
    echo -e "\n#### Used by\n" >> "$markdown_file"
    echo "- [$package_name]( $package_url ) $package_version" >> "$markdown_file"
    echo -e "\n#### License\n" >> "$markdown_file"
    echo '```text' >> "$markdown_file"
    echo -e "$license_description" >> "$markdown_file"
    echo '```' >> "$markdown_file"
done < <(jq -c '.[]' "$json_file")

echo -e "\n## Disclaimer" >> "$markdown_file"
echo -e "
This .NET Third Party Licenses list has been generated with [nuget-license](https://github.com/tomchavakis/nuget-license), \
licensed under [Apache License 2.0](https://github.com/tomchavakis/nuget-license/blob/master/LICENSE)" >> "$markdown_file"

exit 0