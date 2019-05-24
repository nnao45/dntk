#!/bin/sh
sed -n 3p ./Cargo.toml | cut -c 11- | xargs printf