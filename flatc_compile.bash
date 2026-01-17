#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

echo "compiling all flatbuffer schemas in ${SCRIPT_DIR}/etc/*.fbs files to ${SCRIPT_DIR}/gen/flatbuffers/"

cd "${SCRIPT_DIR}"

mkdir -p ${SCRIPT_DIR}/gen/flatbuffers/

cd ${SCRIPT_DIR}/gen/flatbuffers/
"${SCRIPT_DIR}/flatc" --rust "${SCRIPT_DIR}/etc"/*.fbs