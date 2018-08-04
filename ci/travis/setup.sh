#!/bin/sh

set -e

# Find the installed version of a binary, if any
_installed() {
    VERSION=$($@ --version 2>/dev/null || echo "$@ none")
    echo $VERSION | rev | cut -d' ' -f1 | rev
}

# Find the latest available version of a binary on `crates.io`
_latest() {
    VERSION=$(cargo search -q "$@" | grep "$@" | cut -f2 -d"\"")
    echo $VERSION
}

### Setup Rust toolchain #######################################################

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION
export PATH=$PATH:$HOME/.cargo/bin


### Setup kcov #################################################################

if [ ! -f "$HOME/.cargo/bin/kcov" ]; then
    if [ ! -d "$HOME/kcov/.git" ]; then
        git clone --depth=1 https://github.com/SimonKagstrom/kcov "$HOME/kcov"
    fi

    cd $HOME/kcov
    git pull
    cmake .
    make
    install src/kcov $HOME/.cargo/bin/kcov
    cd $TRAVIS_BUILD_DIR
fi

### Setup python linker flags ##################################################

PYTHON_LIB=$(python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"

echo ${LD_LIBRARY_PATH}