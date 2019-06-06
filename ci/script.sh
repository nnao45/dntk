#!/bin/bash
set -ex

export PKG_CONFIG_ALLOW_CROSS=1

cross build --target $TARGET
if [[ "${TARGET}" = "i686-unknown-freebsd" ]] || [[ "${TARGET}" = "x86_64-unknown-freebsd" ]]; then
    echo "'cross test' command is not available for '${TARGET}' target"
    cargo install --force cross
else
    cargo test  --target $TARGET
fi