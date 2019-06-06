#!/bin/bash
set -ex
export PKG_CONFIG_ALLOW_CROSS=1

if [[ "${TARGET}" = "i686-unknown-freebsd" ]] || [[ "${TARGET}" = "x86_64-unknown-freebsd" ]]; then
    cross build --target $TARGET --release
else
    cargo build --target $TARGET --release
fi

mkdir "dntk-${TRAVIS_TAG}-${TARGET}"
cp target/$TARGET/release/dntk LICENSE README.md "dntk-${TRAVIS_TAG}-${TARGET}"

if [[ "${TARGET}" == "i686-pc-windows-msvc" ]] || [[ "${TARGET}" == "x86_64-pc-windows-msvc" ]] ; then
    7z.exe a "dntk-${TRAVIS_TAG}-${TARGET}.zip" "dntk-${TRAVIS_TAG}-${TARGET}"
else
    zip "dntk-${TRAVIS_TAG}-${TARGET}.zip" -r "dntk-${TRAVIS_TAG}-${TARGET}"
fi