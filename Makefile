.PHONY: default build test doc extensions clean cog

ifndef PY
PY := $(word 2, $(subst ., ,$(shell python --version 2>&1)))
endif

ifeq ($(PY),2)
FEATURES := python2
endif
ifeq ($(PY),3)
FEATURES := python3
ifdef PEP384
export PEP384=1
FEATURES := $(FEATURES) pep-384
endif
endif

CARGO_FLAGS := --features "$(FEATURES)" --no-default-features

default: test

build:
	cargo build $(CARGO_FLAGS)

test: build
	cargo test $(CARGO_FLAGS)
	pip install setuptools-rust pytest pytest-benchmark
	cd examples/word-count && python setup.py install && pytest -v tests
	cd examples/word-count-cls && python setup.py install && pytest -v tests

clippy:
	if $$CLIPPY; then cargo clippy $(CARGO_FLAGS); fi


#ifeq ($(NIGHTLY),1)
# ast-json output is only supported on nightly
#	python$(PY) tests/check_symbols.py
#endif

doc: build
	cargo doc --no-deps $(CARGO_FLAGS)

clean:
	rm -r target

gh-pages:
	git clone --branch gh-pages git@github.com:PyO3/PyO3.git gh-pages

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
