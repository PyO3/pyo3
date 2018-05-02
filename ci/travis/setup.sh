#!/bin/sh

### Setup Rust toolchain #######################################################

curl -SsL "https://sh.rustup.rs/" | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION
export PATH=$PATH:$HOME/.cargo/bin


### Setup cargo-make ###########################################################

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


### Setup sccache ##############################################################

export RUSTC_WRAPPER=sccache
mkdir -p $SCCACHE_DIR


### Setup python linker flags ##################################################

python -c """
import sysconfig
cfg = sorted(sysconfig.get_config_vars().items())
print('\n'.join(['{}={}'.format(*x) for x in cfg]))
"""

export PYTHON_LIB=$(python -c "import sysconfig as s; print(s.get_config_var('LIBDIR'))")

# find $PYTHON_LIB
export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"

# delete any possible empty components
# https://github.com/google/pulldown-cmark/issues/122#issuecomment-364948741
LIBRARY_PATH=$(echo $LIBRARY_PATH | sed -E -e 's/^:*//' -e 's/:*$//' -e 's/:+/:/g')

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"
