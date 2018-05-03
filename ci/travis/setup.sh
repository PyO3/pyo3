#!/bin/sh

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

curl -SsL "https://sh.rustup.rs/" | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION
export PATH=$PATH:$HOME/.cargo/bin


### Setup sccache ##############################################################

echo -n "Fetching latest available 'sccache' version... "
INSTALLED=$(_installed sccache)
LATEST=$(_latest sccache)
echo "${LATEST} (installed: ${INSTALLED})"

if [ "$INSTALLED" = "$LATEST" ]; then
  echo "Using cached 'sccache'"
else
  echo "Installing latest 'sccache' from mozilla/sccache"
  URL="https://github.com/mozilla/sccache/releases/download/${LATEST}/sccache-${LATEST}-x86_64-unknown-linux-musl.tar.gz"
  curl -SsL $URL | tar xzv -C /tmp
  mv /tmp/sccache-${LATEST}-x86_64-unknown-linux-musl/sccache $HOME/.cargo/bin/sccache
fi

export SCCACHE_DIR=$HOME/.cargo/sccache
export RUSTC_WRAPPER=sccache
mkdir -p $SCCACHE_DIR


### Setup tarpaulin

echo -n "Fetching latest available 'cargo-tarpaulin' version... "
INSTALLED=$(_installed cargo tarpaulin)
LATEST=$(_latest cargo-tarpaulin)
echo "${LATEST} (installed: ${INSTALLED})"

if [ "$INSTALLED" = "$LATEST" ]; then
  echo "Using cached 'cargo-tarpaulin'"
else
  URL="https://github.com/xd009642/tarpaulin/releases/download/${LATEST}/cargo-tarpaulin-${LATEST}-travis.tar.gz"
  curl -SsL $URL | tar xzvC $HOME/.cargo/bin
fi


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
