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

pip install nox
nox -s build-guide
mv target/guide netlify_build/main/

## Build public docs

nox -s docs
mv target/doc netlify_build/main/doc/

echo "<meta http-equiv=refresh content=0;url=pyo3/>" > netlify_build/main/doc/index.html

## Build internal docs

echo "<div class='internal-banner' style='position:fixed; z-index: 99999; color:red;border:3px solid red;margin-left: auto; margin-right: auto; width: 430px;left:0;right: 0;'><div style='display: flex; align-items: center; justify-content: center;'> ⚠️ Internal Docs ⚠️ Not Public API 👉 <a href='https://pyo3.rs/main/doc/pyo3/index.html' style='color:red;text-decoration:underline;'>Official Docs Here</a></div></div>" > netlify_build/banner.html
RUSTDOCFLAGS="--html-before-content netlify_build/banner.html" nox -s docs -- nightly internal
rm netlify_build/banner.html

mkdir -p netlify_build/internal
mv target/doc netlify_build/internal/

ls -l netlify_build/
