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