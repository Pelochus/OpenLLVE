#!/usr/bin/env bash
# Build and run the OpenLLVE dev container with the repo mounted at
# /workspace.
#
# Usage:
#   docker/run.sh                          # build + run, interactive shell
#   docker/run.sh --cmd './gradlew assembleDebug'   # one-shot command
#   docker/run.sh --skip-build            # use the existing image (e.g. one
#                                         # pulled from GHCR) without rebuilding
#   docker/run.sh --privileged            # extra run options pass through
#
# Runner: CONTAINER_RUNNER=podman|docker (default: podman if installed, else
# docker); the two runners differ in image tagging and in which uid the
# container runs as — see docs/dev/docker-dev-env.md.
#
# In interactive mode the repo's pre-commit hooks are enabled once per
# clone (git config core.hooksPath .githooks), so `git commit` runs
# `cargo fmt --check` and `ktlint` on staged files.
set -euo pipefail

RUNNER=${CONTAINER_RUNNER:-}
if [ -z "$RUNNER" ]; then
	if command -v podman >/dev/null 2>&1; then
		RUNNER=podman
	else
		RUNNER=docker
	fi
fi
case "$RUNNER" in
podman | docker) ;;
*)
	echo "run.sh: CONTAINER_RUNNER must be podman or docker" >&2
	exit 1
	;;
esac

if [ "$RUNNER" = podman ]; then
	IMG=${OPENLLVE_DEV_IMAGE:-localhost/openllve-dev:latest}
	USER_OPTS=()
else
	IMG=${OPENLLVE_DEV_IMAGE:-openllve-dev:latest}
	USER_OPTS=(--user "$(id -u):$(id -g)" --env HOME=/workspace)
fi
REPO="$(cd "$(dirname "$0")/.." && pwd)"

CMD=""
SKIP_BUILD=0
EXTRA=()
args=("$@")
i=0
while [ "$i" -lt "${#args[@]}" ]; do
	case "${args[$i]}" in
	--cmd)
		i=$((i + 1))
		CMD="${args[$i]:?--cmd requires a command}"
		;;
	--skip-build)
		SKIP_BUILD=1
		;;
	*)
		EXTRA+=("${args[$i]}")
		;;
	esac
	i=$((i + 1))
done

if [ "$SKIP_BUILD" -eq 0 ]; then
	"$RUNNER" build -t "$IMG" "$(dirname "$0")"
fi

if [ -n "$CMD" ]; then
	exec "$RUNNER" run --rm -v "$REPO":/workspace "${EXTRA[@]}" "${USER_OPTS[@]}" \
		"$IMG" sh -c "$CMD"
else
	exec "$RUNNER" run --rm -it -v "$REPO":/workspace "${EXTRA[@]}" "${USER_OPTS[@]}" \
		"$IMG" sh -c 'git config core.hooksPath .githooks; exec bash'
fi
