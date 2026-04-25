#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

echo "downloading flatc to ${SCRIPT_DIR}/flatc"

cd "${SCRIPT_DIR}"

git clone https://github.com/google/flatbuffers -b v25.12.19 --depth 1
cd ./flatbuffers
cmake -S . -B ./build -DCMAKE_BUILD_TYPE=Release
cmake --build ./build --config Release -j $(nproc)

cd ./build
./flattests
mv ./flatc "${SCRIPT_DIR}/flatc"

cd "${SCRIPT_DIR}"
sudo rm -r ./flatbuffers