#!/bin/sh

_installed() {
    VERSION=$($@ --version 2>/dev/null || echo "cargo-make none")
    echo $VERSION | cut -d" " -f2
}

_latest() {
    VERSION=$(cargo search -q "$@" | grep "$@" | cut -f2 -d"\"")
    echo $VERSION
}

echo -n "Fetching latest available 'cargo-make' version... "
INSTALLED=$(_installed cargo make)
LATEST=$(_latest cargo-make)
echo "${LATEST} (installed: ${INSTALLED})"

if [ "$INSTALLED" = "$LATEST" ]; then
  echo "Using cached 'cargo-make'"
else
  cargo install -f --debug cargo-make
fi
