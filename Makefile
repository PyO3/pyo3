.PHONY: default build test doc extensions clean

ifndef PY
PY := $(word 2, $(subst ., ,$(shell python --version 2>&1)))
endif
ifndef NIGHTLY
ifeq ($(word 3, $(subst -, ,$(shell rustc --version 2>&1))),nightly)
NIGHTLY := 1
else
NIGHTLY := 0
endif
endif

ifeq ($(PY),2)
FEATURES := python27-sys
endif
ifeq ($(PY),3)
FEATURES := python3-sys
ifdef PEP384
export PEP384=1
FEATURES := $(FEATURES) pep-384
endif
endif
ifeq ($(NIGHTLY),1)
FEATURES := $(FEATURES) nightly
endif

CARGO_FLAGS := --features "$(FEATURES)" --no-default-features

default: test extensions

build:
	cargo build $(CARGO_FLAGS)

test: build
	cargo test $(CARGO_FLAGS)
	python$(PY) tests/check_symbols.py

doc: build
	cargo doc --no-deps $(CARGO_FLAGS)

extensions: build
	make -C extensions/ PY=$(PY)

clean:
	rm -r target
	make -C extensions/ clean

gh-pages:
	git clone --branch gh-pages git@github.com:dgrunwald/rust-cpython.git gh-pages

.PHONY: gh-pages-doc
gh-pages-doc: doc | gh-pages
	cd gh-pages && git pull
	rm -r gh-pages/doc
	cp -r target/doc gh-pages/
	rm gh-pages/doc/.lock
	cd gh-pages && git add .
	cd gh-pages && git commit -m "Update documentation"

publish: default gh-pages-doc
	cargo publish
	cd gh-pages && git push

