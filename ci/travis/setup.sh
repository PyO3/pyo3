#!/bin/sh

set -e

### Setup Rust toolchain #######################################################

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION
export PATH=$PATH:$HOME/.cargo/bin
if [ $TRAVIS_JOB_NAME = 'Minimum nightly' ]; then
    rustup component add clippy
    rustup component add rustfmt
fi

### Setup python linker flags ##################################################

PYTHON_LIB=$(python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"

echo ${LD_LIBRARY_PATH}
