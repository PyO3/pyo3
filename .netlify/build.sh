#!/usr/bin/env bash

set -uex

rustup default nightly

# Install latest mdbook. Netlify will cache the cargo bin dir, so this will
# only build mdbook if needed.
MDBOOK_VERSION=$(cargo search mdbook --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
INSTALLED_MDBOOK_VERSION=$(mdbook --version || echo "none")
if [ "${INSTALLED_MDBOOK_VERSION}" != "mdbook v${MDBOOK_VERSION}" ]; then
    cargo install mdbook@${MDBOOK_VERSION}
fi

pip install nox
nox -s build-guide
cargo xtask doc --internal

mkdir -p netlify_build/internal
mv target/doc netlify_build/internal/
mv target/guide netlify_build/main/

cargo xtask doc
mv target/doc netlify_build/main/doc/

PYO3_VERSION=$(cargo search pyo3 --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
echo "<meta http-equiv=refresh content=0;url=v${PYO3_VERSION}/>" > netlify_build/index.html

# TODO: have some better system to automatically generate this on build rather
# than check this in to the repo
cp .netlify/_redirects netlify_build/

# Add latest redirect
echo "/latest/* https://pyo3.rs/${PYO3_VERSION}/:splat 200" >> netlify_build/_redirects

ls -l netlify_build/

# TODO:
# - netlify badges
# - apply for open source plan
