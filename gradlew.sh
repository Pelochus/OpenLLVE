#!/usr/bin/env sh

# Gradle wrapper script for Unix-based systems.
# This script is used to invoke the Gradle build system.

# Determine the directory of the script
DIR="$(cd "$(dirname "$0")" && pwd)"

# Set the Gradle home directory
GRADLE_HOME="$DIR/gradle"

# Check if Gradle is installed
if [ ! -d "$GRADLE_HOME" ]; then
  echo "Gradle not found. Please install Gradle or set the GRADLE_HOME environment variable."
  exit 1
fi

# Execute Gradle with the provided arguments
exec "$GRADLE_HOME/bin/gradle" "$@"