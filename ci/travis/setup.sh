#!/bin/bash

set -e

### Setup Rust toolchain #######################################################

# Use profile=minimal here to skip installing clippy
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$TRAVIS_RUST_VERSION --profile=minimal -y
export PATH=$PATH:$HOME/.cargo/bin
if [ "$TRAVIS_JOB_NAME" = "Minimum nightly" ]; then
    rustup component add clippy
    rustup component add rustfmt
fi

### Setup PyPy ################################################################

if [[ $FEATURES == *"pypy"* ]]; then
    wget --quiet https://repo.continuum.io/miniconda/Miniconda3-latest-Linux-x86_64.sh && \
    /bin/bash Miniconda3-latest-Linux-x86_64.sh -f -b -p /opt/anaconda && \
    /opt/anaconda/bin/conda install --quiet --yes conda && \
    /opt/anaconda/bin/conda config --system --add channels conda-forge && \
    /opt/anaconda/bin/conda init bash && \
    /opt/anaconda/bin/conda create -n pypy3 pypy3.5 -y && \
    /opt/anaconda/envs/pypy3/bin/pypy3 -m ensurepip && \
    /opt/anaconda/envs/pypy3/bin/pypy3 -m pip install setuptools-rust pytest pytest-benchmark tox
fi

### Setup python linker flags ##################################################

if [[ $FEATURES == *"pypy"* ]]; then
    PYTHON_BINARY="pypy3"
else
    PYTHON_BINARY="python"
fi

PYTHON_LIB=$($PYTHON_BINARY -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"

echo ${LD_LIBRARY_PATH}
