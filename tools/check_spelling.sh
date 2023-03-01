#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

if [ $# -ne 1 ]; then
  echo "Usage: $0 {filename}"
  exit 1
fi

TEMP_FILE=`mktemp`
TARGET_DIR=`dirname $1`
ACCEPTED_WORDS_FILEPATH="$TARGET_DIR/.accepted_words.txt"

if [ -e $ACCEPTED_WORDS_FILEPATH ]; then
  spell -d $ACCEPTED_WORDS_FILEPATH $1 | sort -u > $TEMP_FILE
else
  spell $1 | sort -u > $TEMP_FILE
fi
NUM_SPELLING_ERRORS=`wc -l $TEMP_FILE | cut -d ' ' -f 1`

if [ $NUM_SPELLING_ERRORS -eq 0 ]; then
  echo "$1 has no spelling mistakes."
else
  echo "Error: $1 has spelling mistakes. Please fix them."
  cat $TEMP_FILE
fi

rm -f $TEMP_FILE

exit $NUM_SPELLING_ERRORS
