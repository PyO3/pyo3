#!/usr/bin/env bash

set -uex

rustup default nightly

PYO3_VERSION=$(cargo search pyo3 --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')

## Start from the existing gh-pages content.
## By serving it over netlify, we can have better UX for users because
## netlify can then redirect e.g. /v0.17.0 to /v0.17.0/
## which leads to better loading of CSS assets.

wget -qc https://github.com/PyO3/pyo3/archive/gh-pages.tar.gz -O - | tar -xz
mv pyo3-gh-pages netlify_build

## Configure netlify _redirects file

# Add redirect for each documented version
for d in netlify_build/v*; do
    version="${d/netlify_build\/v/}"
    echo "/v$version/doc/* https://docs.rs/pyo3/$version/:splat" >> netlify_build/_redirects
done

# Add latest redirect
echo "/latest/* /v${PYO3_VERSION}/:splat" >> netlify_build/_redirects

## Add landing page redirect
if [ "${CONTEXT}" == "deploy-preview" ]; then
    echo "/ /main/" >> netlify_build/_redirects
else
    echo "/ /v${PYO3_VERSION}/" >> netlify_build/_redirects
fi

## Generate towncrier release notes

pip install towncrier
towncrier build --yes --version Unreleased --date TBC

## Build guide

# Install latest mdbook. Netlify will cache the cargo bin dir, so this will
# only build mdbook if needed.
MDBOOK_VERSION=$(cargo search mdbook --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
INSTALLED_MDBOOK_VERSION=$(mdbook --version || echo "none")
if [ "${INSTALLED_MDBOOK_VERSION}" != "mdbook v${MDBOOK_VERSION}" ]; then
    cargo install mdbook@${MDBOOK_VERSION} --force
fi

pip install nox
nox -s build-guide
mv target/guide netlify_build/main/

## Build public docs

cargo xtask doc
mv target/doc netlify_build/main/doc/

echo "<meta http-equiv=refresh content=0;url=pyo3/>" > netlify_build/main/doc/index.html

## Build internal docs

echo "<div class='internal-banner' style='position:fixed; z-index: 99999; color:red;border:3px solid red;margin-left: auto; margin-right: auto; width: 430px;left:0;right: 0;'><div style='display: flex; align-items: center; justify-content: center;'> ⚠️ Internal Docs ⚠️ Not Public API 👉 <a href='https://pyo3.rs/main/doc/pyo3/index.html' style='color:red;text-decoration:underline;'>Official Docs Here</a></div></div>" > netlify_build/banner.html
RUSTDOCFLAGS="--html-before-content netlify_build/banner.html" cargo xtask doc --internal
rm netlify_build/banner.html

mkdir -p netlify_build/internal
mv target/doc netlify_build/internal/

ls -l netlify_build/
