#!/bin/sh
SEMATIC_VERSION_REGEX='^([0-9]+)\.([0-9]+)\.([0-9]+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z-]+)?$'
printf 'release version?: '
read ANSWER
if ! printf ${ANSWER} | egrep ${SEMATIC_VERSION_REGEX} >/dev/null 2>&1; then
    echo 'Sorry, This is no sematic version, retry'
    exit 1
fi
echo "release ${ANSWER}"
sed -i '' -e "s/version = \"$(./version.sh | cut -c 2-)\"/version = \"${ANSWER}\"/g" Cargo.toml
sleep 3