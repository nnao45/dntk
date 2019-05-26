#!/bin/sh
sed -n 3p ./Cargo.toml | cut -c 11- | sed '1s/^/v/' | xargs printf