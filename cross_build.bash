#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}"
mkdir -p ./cross

if [[ "$#" -ne "3" ]]; then
    echo "Please provide 3 arguments in the form of: TARGET TARGET-SSH TARGET-SOURCE-LOCATION"
    exit 1
fi
TARGET=$1
# set up your ssh for public key authentication if you don't want to have to type in passwords
TARGET_SSH=$2
TARGET_SOURCE_LOCATION=$3

echo "syncing environment from ${TARGET_SSH}"
rsync --archive --sparse --compress --fsync --delete --recursive --mkpath --quiet --checksum "${TARGET_SSH}:/${TARGET_SOURCE_LOCATION}/lib" "${SCRIPT_DIR}/cross"

echo "building"
# `cargo install cross` if cross is not avaliable (you also need to have docker avaliable)
cross build --target "${TARGET}" --no-default-features

echo "deploying"
cd "${SCRIPT_DIR}/target/${TARGET}"

# this ignores anything in */deps/* (all the stuff generated in there binaries but not actually useful for running)
find ./ -type f ! -path "*/deps/*" -executable -print0 \
| rsync --archive --sparse --compress --fsync --delete --recursive --mkpath --progress --checksum --files-from=- --from0 ./ "${TARGET_SSH}:/${TARGET_SOURCE_LOCATION}/target/deployed"

