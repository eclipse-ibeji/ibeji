#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e
cd "$(dirname "$0")/.."

if ! command -v gh &> /dev/null
then
    echo "GitHub CLI not found. Please install before running the script."
    exit
fi

if [ -z "$GITHUB_TOKEN" ]
then
      echo "Missing \$GITHUB_TOKEN environment variable. Please set it before running the script."
      exit 1
fi

if ! command -v cargo-about &> /dev/null
then
    echo "Cargo-about could not be found. Installing now"
    cargo install --locked cargo-about
fi

PR_TITLE="chore: Notice file change"
if [ `gh pr list --search "$PR_TITLE" --json number | jq '. | length'` -gt 0 ]
then
    echo>&2 "A PR is already there for a NOTICE file change. Please merge it or cancel it to have this pipeline properly running."
    exit 1
fi

NOTICE_FILENAME="NOTICE"
echo "Running cargo-about for NOTICE file generation..."
cargo about generate --workspace devops/cg/about.hbs --config devops/cg/about.toml > $NOTICE_FILENAME

DOTNET_SRC_DIRECTORY="dtdl-tools/"
echo "Appending .NET Third Party licenses to $NOTICE_FILENAME"
./tools/dotnet_notice_generation.sh $NOTICE_FILENAME $DOTNET_SRC_DIRECTORY ./devops/cg/license_url_to_type.json

if [ -z "$(git diff --name-only $NOTICE_FILENAME)" ]
then
      echo "File not changed"
else
      echo "File changed. Checking out a new branch and creating a PR"
      BRANCH_NAME="fix/notice-file-update-$(date +%s)"
      git checkout -b "$BRANCH_NAME"
      git add $NOTICE_FILENAME
      git commit -m "New notice file"
      git push -f --set-upstream origin "$BRANCH_NAME"
      gh pr create -B main -H "$BRANCH_NAME" --title "$PR_TITLE" --body 'This PR is merging latest changes related to notice file. Please review them before approving.'
fi
