#!/usr/bin/env bash
# Add an external model repository as a git submodule under external/models/.
#
# Usage:   scripts/add-model-submodule.sh <name> <repo-url>
# Example: scripts/add-model-submodule.sh mblLEN https://github.com/Lvfeifan/MBLLEN
#
# The script also creates a symlink to the model's .tflite in
# app/platforms/android/src/main/assets/models/ so the app can load it from
# assets (see external/models/README.md).
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <name> <repo-url>" >&2
  exit 2
fi

name=$1
url=$2

if [ -e "external/models/$name" ] || [ -L "external/models/$name" ]; then
  echo "error: external/models/$name already exists" >&2
  exit 1
fi

git submodule add "$url" "external/models/$name"

# Pick the first .tflite in the submodule (path order, deterministic).
model_file=$(find "external/models/$name" -name '*.tflite' -not -path '*/.git/*' | sort | head -n 1)
if [ -z "$model_file" ]; then
  echo "error: no .tflite found in external/models/$name" >&2
  exit 1
fi

assets_dir="app/platforms/android/src/main/assets/models"
link="$assets_dir/$(basename "$model_file")"
if [ -e "$link" ] || [ -L "$link" ]; then
  echo "error: $link already exists" >&2
  exit 1
fi

# The assets dir is 7 levels below the repo root.
ln -s "../../../../../..../external/models/${name}/$(basename "$model_file")" "$link"

echo "Submodule added at external/models/$name."
echo "Symlink created: $link"
echo "Next: validate the model upstream, then record its I/O shape and license in $assets_dir/README.md."
