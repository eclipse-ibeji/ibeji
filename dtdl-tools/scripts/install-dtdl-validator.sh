#!/bin/bash

set -e

# Default values
src_dir="."
dst_dir="$HOME/.cargo/bin/dtdl-validator"

# Parse command line options
while getopts "s:d:" opt; do
  case ${opt} in
    s)
      src_dir="$OPTARG"
      ;;
    d)
      dst_dir="$OPTARG"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" 1>&2
      echo "Usage: $0 [-s source_directory] [-d destination_directory]" 1>&2
      exit 1
      ;;
  esac
done

# Check if dtdl-validator exists in the source directory
if [[ ! -f "$src_dir/dtdl-validator" ]]; then
  echo "Error: dtdl-validator must exist in the source directory." 1>&2
  exit 1
fi

# Create the destination directory if it does not exist
mkdir -p "$dst_dir"

# Copy dtdl-validator, its config file and all assocaited dll files to the destination directory
cp "$src_dir/dtdl-validator" "$dst_dir"
cp "$src_dir"/dtdl-validator.runtimeconfig.json "$dst_dir"
cp "$src_dir"/*.dll "$dst_dir"

echo "dtdl-validator has been successfully installed in $dst_dir"

exit 0
