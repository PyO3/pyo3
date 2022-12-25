#!/usr/bin/env python
"""Generates html files for netlify which add forward-slashes
onto redirects.

This ensures that the guide CSS works nicely when being served
over netlify.
"""

from pathlib import Path

NETLIFY_DIR = Path(__file__).parent
NETLIFY_BUILD_DIR = NETLIFY_DIR.parent / "netlify_build"


def main() -> None:
    redirects_file = NETLIFY_DIR / "_redirects"
    for line in redirects_file.read_text().splitlines():
        redirect = line.split()[0]
        if not (redirect.endswith("/*") and line.endswith("200!")):
            # not a glob rewrite, no need to make forward-slash addition
            continue
        # remove leading / and trailing /*, add .html to final path segment
        path = redirect[1:-2].split("/")
        path[-1] += ".html"

        target_file = NETLIFY_BUILD_DIR.joinpath(*path)
        target_file.parent.mkdir(exist_ok=True, parents=True)

        # location to redirect to is the redirect without the glob
        target_redirect = redirect.rstrip("*")
        target_file.write_text(
            f"<meta http-equiv=refresh content=0;url={target_redirect}/>"
        )


if __name__ == "__main__":
    main()
