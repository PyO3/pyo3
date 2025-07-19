#!/usr/bin/env bash

set -uex

rustup update nightly
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
set +x  # these loops get very spammy and fill the deploy log

for d in netlify_build/v*; do
    version="${d/netlify_build\/v/}"
    echo "/v$version/doc/* https://docs.rs/pyo3/$version/:splat" >> netlify_build/_redirects
    if [ $version != $PYO3_VERSION ]; then
        # for old versions, mark the files in the latest version as the canonical URL
        for file in $(find $d -type f); do
            file_path="${file/$d\//}"
            # remove index.html and/or .html suffix to match the page URL on the
            # final netlfiy site
            url_path="$file_path"
            if [[ $file_path == index.html ]]; then
                url_path=""
            elif [[ $file_path == *.html ]]; then
                url_path="${file_path%.html}"
            fi
            echo "/v$version/$url_path" >> netlify_build/_headers
            if test -f "netlify_build/v$PYO3_VERSION/$file_path"; then
                echo "  Link: <https://pyo3.rs/v$PYO3_VERSION/$url_path>; rel=\"canonical\"" >> netlify_build/_headers
            else
                # this file doesn't exist in the latest guide, don't index it
                echo "  X-Robots-Tag: noindex" >> netlify_build/_headers
            fi
        done
    fi
done

# Add latest redirect
echo "/latest/* /v${PYO3_VERSION}/:splat 302" >> netlify_build/_redirects

# some backwards compatbiility redirects
echo "/latest/building_and_distribution/* /latest/building-and-distribution/:splat 302" >> netlify_build/_redirects
echo "/latest/building-and-distribution/multiple_python_versions/* /latest/building-and-distribution/multiple-python-versions:splat 302" >> netlify_build/_redirects
echo "/latest/function/error_handling/* /latest/function/error-handling/:splat 302" >> netlify_build/_redirects
echo "/latest/getting_started/* /latest/getting-started/:splat 302" >> netlify_build/_redirects
echo "/latest/python_from_rust/* /latest/python-from-rust/:splat 302" >> netlify_build/_redirects
echo "/latest/python_typing_hints/* /latest/python-typing-hints/:splat 302" >> netlify_build/_redirects
echo "/latest/trait_bounds/* /latest/trait-bounds/:splat 302" >> netlify_build/_redirects

## Add landing page redirect
if [ "${CONTEXT}" == "deploy-preview" ]; then
    echo "/ /main/" >> netlify_build/_redirects
else
    echo "/ /v${PYO3_VERSION}/ 302" >> netlify_build/_redirects
fi

set -x
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

# Install latest mdbook-linkcheck. Netlify will cache the cargo bin dir, so this will
# only build mdbook-linkcheck if needed.
MDBOOK_LINKCHECK_VERSION=$(cargo search mdbook-linkcheck --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
INSTALLED_MDBOOK_LINKCHECK_VERSION=$(mdbook-linkcheck --version || echo "none")
if [ "${INSTALLED_MDBOOK_LINKCHECK_VERSION}" != "mdbook v${MDBOOK_LINKCHECK_VERSION}" ]; then
    cargo install mdbook-linkcheck@${MDBOOK_LINKCHECK_VERSION} --force
fi

# Install latest mdbook-tabs. Netlify will cache the cargo bin dir, so this will
# only build mdbook-tabs if needed.
MDBOOK_TABS_VERSION=$(cargo search mdbook-tabs --limit 1 | head -1 | tr -s ' ' | cut -d ' ' -f 3 | tr -d '"')
INSTALLED_MDBOOK_TABS_VERSION=$(mdbook-tabs --version || echo "none")
if [ "${INSTALLED_MDBOOK_TABS_VERSION}" != "mdbook-tabs v${MDBOOK_TABS_VERSION}" ]; then
    cargo install mdbook-tabs@${MDBOOK_TABS_VERSION} --force
fi

pip install nox
nox -s build-guide
mv target/guide/ netlify_build/main/

## Build public docs

nox -s docs
mv target/doc netlify_build/main/doc/

echo "<meta http-equiv=refresh content=0;url=pyo3/>" > netlify_build/main/doc/index.html

## Build internal docs

nox -s docs -- nightly internal

mkdir -p netlify_build/internal
mv target/doc netlify_build/internal/

ls -l netlify_build/
