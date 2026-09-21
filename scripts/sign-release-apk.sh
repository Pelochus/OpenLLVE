#!/usr/bin/env bash
# Sign the release APK with your own keystore.
#
# This script is the single source of truth for release signing: it sets the
# RELEASE_* environment variables consumed by app/build.gradle.kts, then
# builds the signed release APK.
#
# Usage:
#   scripts/sign-release-apk.sh <keystore> <key-alias> <key-password> [store-password]
#
# If [store-password] is omitted it defaults to <key-password>.
#
# Create a keystore first (one-time):
#   keytool -genkeypair -v -keystore openllve.keystore -alias openllve \
#     -dname "CN=OpenLLVE" -keyalg RSA -keysize 2048 -validity 10000 \
#     -storepass <pass> -keypass <pass>
#
# Then sign:
#   scripts/sign-release-apk.sh openllve.keystore openllve <pass>
#
# Output: app/build/outputs/apk/release/app-release.apk
set -euo pipefail

usage="usage: sign-release-apk.sh <keystore> <key-alias> <key-password> [store-password]"
keystore=${1:?$usage}
alias=${2:?$usage}
key_pass=${3:?$usage}
store_pass=${4:-$key_pass}

if [[ ! -f "$keystore" ]]; then
	echo "error: keystore not found: $keystore" >&2
	exit 1
fi

if [[ -z "${JAVA_HOME:-}" ]] && ! command -v java >/dev/null 2>&1; then
	echo "error: no JDK found. Set JAVA_HOME or put java on PATH." >&2
	exit 1
fi

keystore_abs="$(readlink -f "$keystore")"
export RELEASE_KEYSTORE="$keystore_abs"
export RELEASE_KEY_ALIAS="$alias"
export RELEASE_KEY_PASSWORD="$key_pass"
export RELEASE_STORE_PASSWORD="$store_pass"

./gradlew :app:assembleRelease --console=plain

echo
echo "Signed release APK: app/build/outputs/apk/release/app-release.apk"

sdk_dir="$(sed -n 's/^sdk\.dir=//p' local.properties 2>/dev/null)"
apksigner="$(ls -d "$sdk_dir"/build-tools/*/apksigner 2>/dev/null | sort -V | tail -1)"
if [[ -n "$apksigner" ]]; then
	echo "Verify with: $apksigner verify --print-certs app/build/outputs/apk/release/app-release.apk"
fi
