#!/bin/bash

set -e

### Setup Rust toolchain #######################################################

# Use profile=minimal here to skip installing clippy
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$TRAVIS_RUST_VERSION --profile=minimal -y
export PATH=$PATH:$HOME/.cargo/bin
if [[ $RUN_LINT == 1 ]]; then
    rustup component add clippy
    rustup component add rustfmt
fi

### Setup python linker flags ##################################################
PYTHON_LIB=$(python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"

echo ${LD_LIBRARY_PATH}
