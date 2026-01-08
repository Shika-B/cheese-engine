#!/bin/sh

set -e

NAME="$1"

if [ -z "$NAME" ]; then
    echo "Usage: $0 <name>"
    exit 1
fi

mkdir -p executables
cp target/release/cheese-engine "executables/$NAME"

echo "Copied cheese-engine to executables/$NAME"
