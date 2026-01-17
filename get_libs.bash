#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}/lib"

echo "Downloading LR11XX Driver"
git clone https://github.com/Lora-net/SWDR001
cd SWDR001
git checkout f99fe41538e351c4c0d1975a4138532fe7869d65
echo "Building LR11XX Driver"
cmake -S . -B ./build
cmake --build ./build --config Release -j $(nrpoc)
echo "LR11XX driver built"

cd "${SCRIPT_DIR}/lib"

echo "Downloading sx1302 driver"
git clone https://github.com/Lora-net/sx1302_hal
cd sx1302_hal
git checkout 4b42025d1751e04632c0b04160e0d29dbbb222a5
echo "Building SX1302 Driver"
make clean all -j $(nproc)
echo "SX1302 driver built"

echo ""
echo "all tasks finished, all source libraries installed and built."