.PHONY: default test publish

ifndef PY
PY := $(word 2, $(subst ., ,$(shell python --version 2>&1)))
endif

ifeq ($(PY),2)
FEATURES := python2
endif
ifeq ($(PY),3)
FEATURES := python3
endif

CARGO_FLAGS := --features "$(FEATURES)" --no-default-features

default: test

test:
	cargo test $(CARGO_FLAGS)
	pip install tox
	tox
	cd examples/word-count && tox
	cd examples/rustapi_module && tox

publish:
	cargo publish --manifest-path pyo3-derive-backend/Cargo.toml
	cargo publish --manifest-path pyo3cls/Cargo.toml
	cargo publish
