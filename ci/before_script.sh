#!/bin/bash
set -ex

rustc -V
cargo -V
# if [[ "${TARGET}" == "x86_64-unknown-linux-musl" ]] || [[ "${TARGET}" == "x86_64-pc-windows-msvc" ]]; then rustup target add $TARGET; fi
rustup target add $TARGET

if [[ "${TARGET}" == "x86_64-pc-windows-msvc" ]] || [[ "${TARGET}" == "i686-pc-windows-msvc" ]] ; then
    wget https://embedeo.org/ws/command_line/bc_dc_calculator_windows/bc-1.07.1-win32-embedeo-02.zip
    unzip bc-1.07.1-win32-embedeo-02.zip
    export PATH=$PATH:"/c/Users/travis/build/nnao45/dntk/bc-1.07.1-win32-embedeo-02/bin"
fi

#if [[ "${TARGET}" == "aarch64-unknown-linux-gnu" ]]; then
#    aarch64-linux-gnu-gcc -v
#    mkdir -p .cargo
#    echo "[target.${TARGET}]" > .cargo/config
#    echo "linker = \"aarch64-linux-gnu-gcc\"" >> .cargo/config
#    cat .cargo/config
#fi

git --version
echo $TRAVIS_BRANCH
pwd
echo $PATH
git rev-parse HEAD