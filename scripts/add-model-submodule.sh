#!/usr/bin/env bash
# Add an external model repository as a git submodule under external/models/.
#
# Usage:   scripts/add-model-submodule.sh <name> <repo-url>
# Example: scripts/add-model-submodule.sh mblLEN https://github.com/Lvfeifan/MBLLEN
#
# After adding, copy the .tflite into
# app/platforms/android/src/main/assets/models/ and commit it (see external/models/README.md).
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <name> <repo-url>" >&2
  exit 2
fi

name=$1
url=$2

if [ -e "external/models/$name" ]; then
  echo "error: external/models/$name already exists" >&2
  exit 1
fi

git submodule add "$url" "external/models/$name"

echo "Submodule added at external/models/$name."
echo "Next: validate the model upstream, then copy its .tflite into"
echo "      app/platforms/android/src/main/assets/models/ and commit."
