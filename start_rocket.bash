#! /bin/bash

# This script automatically chooses a rocket side binary/main and config to run
# You may specify through cli arguments what config and binary should be ran
#
# Usage: ./start_rocket.bash [config_path?] [binary_path?] 
#
# The search order for config files is:
#       0. CLI argument 1
#       1. ./etc/config.toml
#
# The search order for binaries to use is:
#       0. CLI argument 2
#       1. ./target/deployed/main
#       2. ./target/release/main
#       3. ./target/debug/main
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
elif [[ -f "./target/deployed/release/main" ]]; then
    echo "[CLI] Deployed release build found! Using ./target/deployed/release/main"
    TARGET="./target/deployed/release/main"
elif [[ -f "./target/deployed/debug/main" ]]; then
    echo "[CLI] Deployed debug build found! Using ./target/deployed/debug/main"
    TARGET="./target/deployed/debug/main"
elif [[ -f "./target/release/main" ]]; then
    echo "[CLI] Local release main build found! Using ./target/release/main"
    TARGET="./target/release/main"
elif [[ -f "./target/debug/main" ]]; then
    echo "[CLI] Local debug main build found! Using ./target/debug/main"
    TARGET="./target/debug/main"
fi
if [[ ! -f "$TARGET" ]]; then
    echo "[CLI] No compiled binaries found! Unable to start!"
    exit 1
fi

echo "[CLI] Starting main using binary at $TARGET and config at $CONFIG"
exec $TARGET "$CONFIG"