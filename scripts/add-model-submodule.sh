#!/usr/bin/env bash
# Add an external model repository as a git submodule under models/.
#
# Usage:   scripts/add-model-submodule.sh <name> <repo-url>
# Example: scripts/add-model-submodule.sh mblLEN https://github.com/Lvfeifan/MBLLEN
#
# After adding, copy the .tflite into
# app/platforms/android/src/main/assets/models/ and commit it (see models/README.md).
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <name> <repo-url>" >&2
  exit 2
fi

name=$1
url=$2

if [ -e "models/$name" ]; then
  echo "error: models/$name already exists" >&2
  exit 1
fi

git submodule add "$url" "models/$name"

echo "Submodule added at models/$name."
echo "Next: validate the model upstream, then copy its .tflite into"
echo "      app/platforms/android/src/main/assets/models/ and commit."
