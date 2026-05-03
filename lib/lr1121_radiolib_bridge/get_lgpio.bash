#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}"
mkdir -p ./build
cd ./build

echo "Downloading lgpio"
git clone https://github.com/joan2937/lg
cd lg
git checkout bcccd782eceedc5b278b3056ea81d5fbbb89c489