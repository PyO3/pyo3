#!/usr/bin/env python
"""Generates _redirects file for netlify.

Run this and write output to .netlify/_redirects and check into
the PyO3 repository.
"""

import subprocess


def main() -> None:
    versions = subprocess.check_output(["git", "tag"], text=True).splitlines()
    for version in versions:
        version_without_v = version.lstrip("v")
        # redirect doc requests to docs.rs
        print(f"/{version}/doc/* https://docs.rs/pyo3/{version_without_v}/:splat")
        # proxy guide to github-pages hosting
        print(f"/{version}/* https://pyo3.github.io/pyo3/{version}/:splat 200")
    # proxy benchmarks to github-pages hosting
    print(f"/dev/bench/* https://pyo3.github.io/pyo3/dev/bench/:splat 200")


if __name__ == "__main__":
    main()
