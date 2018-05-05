#!/bin/sh

set -e

### Setup latest mdbook version ################################################

INSTALLED=$(echo $(mdbook --version 2>/dev/null || echo "mdbook none") | cut -d' ' -f1)
LATEST=0.1.5

if [ "$LATEST" != "$INSTALLED" ]; then
    URL=https://github.com/rust-lang-nursery/mdBook/releases/download/v${LATEST}/mdbook-v${LATEST}-x86_64-unknown-linux-gnu.tar.gz
    curl -SsL $URL | tar xvz -C $HOME/.cargo/bin
fi

### Build API reference ########################################################

cargo doc --no-deps -p pyo3 -p pyo3cls
echo "<meta http-equiv=refresh content=0;url='guide/'>" > target/doc/index.html


### Build guide ################################################################

cd guide
mdbook build -d ../target/doc/guide
cd ..

git clone https://github.com/davisp/ghp-import.git
./ghp-import/ghp_import.py -n -p -f -m "Documentation upload" -r https://"$GH_TOKEN"@github.com/"$TRAVIS_REPO_SLUG.git" target/doc
echo "Uploaded documentation"
