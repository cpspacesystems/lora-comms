#! /bin/bash

# This script automatically chooses a ground station binary and config to run
# You may specify through cli arguments what config and binary should be ran
#
# Usage: ./start_ground.bash [config_path?] [binary_path?] 
#
# The search order for config files is:
#       0. CLI argument 1
#       1. ./etc/config.toml
#
# The search order for binaries to use is:
#       0. CLI argument 2
#       1. ./target/deployed/ground
#       2. ./target/release/ground
#       3. ./target/debug/ground
#
# Search orders goes from lowest to highest. With the lowest ordered path(ie: 0)
#   being searched first. Then proceeding on if the path do not exist.

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}"

CONFIG="./etc/config.toml"
if [[ ! -z "$1" ]]; then
    CONFIG="$1"
    echo "[CLI] Using command line specified config at $CONFIG"
fi
if [[ ! -f "$CONFIG" ]]; then
    echo "[CLI] No config found! Unable to start!"
    exit 1
fi

TARGET=""
if [[ ! -z "$2" ]]; then
    TARGET=$2
    echo "[CLI] Using command line specified binary at $TARGET"
elif [[ -f "./target/deployed/release/ground" ]]; then
    echo "[CLI] Deployed release ground build found! Using ./target/deployed/release/ground"
    TARGET="./target/deployed/release/ground"
elif [[ -f "./target/deployed/debug/ground" ]]; then
    echo "[CLI] Deployed debug ground build found! Using ./target/deployed/debug/ground"
    TARGET="./target/deployed/debug/ground"
elif [[ -f "./target/release/ground" ]]; then
    echo "[CLI] Local release ground build found! Using ./target/release/ground"
    TARGET="./target/release/ground"
elif [[ -f "./target/debug/ground" ]]; then
    echo "[CLI] Local debug ground build found! Using ./target/debug/ground"
    TARGET="./target/debug/ground"
fi
if [[ ! -f "$TARGET" ]]; then
    echo "[CLI] No compiled binaries found! Unable to start!"
    exit 1
fi

echo "[CLI] Resetting LoRa gateway."
./reset_lgw.sh stop
echo "[CLI] Starting LoRa gateway."
./reset_lgw.sh start

echo "[CLI] Starting ground using binary at $TARGET and config at $CONFIG"
exec $TARGET "$CONFIG"