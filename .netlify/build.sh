#!/usr/bin/env bash

set -uex

rustup default nightly

PYO3_VERSION=$(cargo search pyo3 --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
mkdir netlify_build

## Configure netlify _redirects file

# TODO: have some better system to automatically generate this on build rather
# than check this in to the repo
cp .netlify/_redirects netlify_build/

# Add latest redirect (proxy)
echo "/latest/* https://pyo3.rs/v${PYO3_VERSION}/:splat 200" >> netlify_build/_redirects

## Add landing page redirect
echo "<meta http-equiv=refresh content=0;url=v${PYO3_VERSION}/>" > netlify_build/index.html


## Build guide

# Install latest mdbook. Netlify will cache the cargo bin dir, so this will
# only build mdbook if needed.
MDBOOK_VERSION=$(cargo search mdbook --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
INSTALLED_MDBOOK_VERSION=$(mdbook --version || echo "none")
if [ "${INSTALLED_MDBOOK_VERSION}" != "mdbook v${MDBOOK_VERSION}" ]; then
    cargo install mdbook@${MDBOOK_VERSION}
fi

pip install nox
nox -s build-guide
mv target/guide netlify_build/main/

## Build public docs

cargo xtask doc
mv target/doc netlify_build/main/doc/

## Build internal docs

echo "<div class='internal-banner' style='position:fixed; z-index: 99999; color:red;border:3px solid red;margin-left: auto; margin-right: auto; width: 430px;left:0;right: 0;'><div style='display: flex; align-items: center; justify-content: center;'> ⚠️ Internal Docs ⚠️ Not Public API 👉 <a href='https://pyo3.rs/main/doc/pyo3/index.html' style='color:red;text-decoration:underline;'>Official Docs Here</a></div></div>" > netlify_build/banner.html
RUSTDOCFLAGS="--html-before-content netlify_build/banner.html" cargo xtask doc --internal
rm netlify_build/banner.html

mkdir -p netlify_build/internal
mv target/doc netlify_build/internal/

ls -l netlify_build/

# TODO:
# - netlify badges
# - apply for open source plan
