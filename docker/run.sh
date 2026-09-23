#!/usr/bin/env bash
# Build and run the OpenLLVE dev container with the repo mounted at
# /workspace.
#
# Usage:
#   docker/run.sh                          # build + run, interactive shell
#   docker/run.sh --privileged            # extra run options pass through
#   docker/run.sh --cmd './gradlew assembleDebug'   # one-shot command
#
# In interactive mode the repo's pre-commit hooks are enabled once per
# clone (git config core.hooksPath .githooks), so `git commit` runs
# `cargo fmt --check` and `ktlint` on staged files.
set -euo pipefail

RUNNER=${CONTAINER_RUNNER:-podman}
# Fully-qualified name: some podman versions treat bare short names as
# needing registry resolution (and prompt, which fails without a TTY).
IMG=${OPENLLVE_DEV_IMAGE:-localhost/openllve-dev:latest}
REPO="$(cd "$(dirname "$0")/.." && pwd)"

CMD=""
EXTRA=()
args=("$@")
i=0
while [ "$i" -lt "${#args[@]}" ]; do
	case "${args[$i]}" in
	--cmd)
		i=$((i + 1))
		CMD="${args[$i]:?--cmd requires a command}"
		;;
	*)
		EXTRA+=("${args[$i]}")
		;;
	esac
	i=$((i + 1))
done

"$RUNNER" build -t "$IMG" "$(dirname "$0")"

if [ -n "$CMD" ]; then
	exec "$RUNNER" run --rm -v "$REPO":/workspace "${EXTRA[@]}" "$IMG" \
		sh -c "$CMD"
else
	exec "$RUNNER" run --rm -it -v "$REPO":/workspace "${EXTRA[@]}" "$IMG" \
		sh -c 'git config core.hooksPath .githooks; exec bash'
fi
