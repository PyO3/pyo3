"""Simple mdbook preprocessor to inject pyo3 version into the guide.

It will replace:
    - {{#PYO3_VERSION_TAG}} with the contents of the PYO3_VERSION_TAG environment var
    - {{#PYO3_DOCS_URL}} with the location of docs (e.g. 'https://docs.rs/pyo3/0.13.2')
    - {{#PYO3_CRATE_VERSION}} with a relevant toml snippet (e.g. 'version = "0.13.2"')


Tested against mdbook 0.4.10.
"""

import json
import os
import sys

# Set PYO3_VERSION in CI to build the correct version into links
PYO3_VERSION_TAG = os.environ.get("PYO3_VERSION_TAG", "main")

if PYO3_VERSION_TAG == "main":
    PYO3_DOCS_URL = "https://pyo3.rs/main/doc"
    PYO3_DOCS_VERSION = "latest"
    PYO3_CRATE_VERSION = 'git = "https://github.com/pyo3/pyo3"'
else:
    # v0.13.2 -> 0.13.2
    version = PYO3_VERSION_TAG.lstrip("v")
    PYO3_DOCS_URL = f"https://docs.rs/pyo3/{version}"
    PYO3_DOCS_VERSION = version
    PYO3_CRATE_VERSION = f'version = "{version}"'


def replace_section_content(section):
    if not isinstance(section, dict) or "Chapter" not in section:
        return

    # Replace raw and url-encoded forms
    section["Chapter"]["content"] = (
        section["Chapter"]["content"]
        .replace("{{#PYO3_VERSION_TAG}}", PYO3_VERSION_TAG)
        .replace("{{#PYO3_DOCS_URL}}", PYO3_DOCS_URL)
        .replace("{{#PYO3_DOCS_VERSION}}", PYO3_DOCS_VERSION)
        .replace("{{#PYO3_CRATE_VERSION}}", PYO3_CRATE_VERSION)
    )

    for sub_item in section["Chapter"]["sub_items"]:
        replace_section_content(sub_item)


for line in sys.stdin:
    if line:
        [context, book] = json.loads(line)
        for section in book["sections"]:
            replace_section_content(section)
        json.dump(book, fp=sys.stdout)
