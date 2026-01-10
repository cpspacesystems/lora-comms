#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

echo "downloading flatc to ${SCRIPT_DIR}/flatc"

cd "${SCRIPT_DIR}"

wget https://github.com/google/flatbuffers/releases/download/v25.12.19/Linux.flatc.binary.g++-13.zip
unzip Linux.flatc.binary.g++-13.zip
rm Linux.flatc.binary.g++-13.zip