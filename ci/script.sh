#!/bin/bash
set -ex
export PKG_CONFIG_ALLOW_CROSS=1

if [[ "${TARGET}" = "i686-unknown-freebsd" ]] || [[ "${TARGET}" = "x86_64-unknown-freebsd" ]]; then
    cargo install --force cross
    cross clippy --target=${TARGET} -- -D warnings
else
    cargo test  --target $TARGET
    cargo clippy --target $TARGET -- -D warnings
fi