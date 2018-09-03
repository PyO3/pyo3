.PHONY: default test

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
	pip install setuptools-rust pytest pytest-benchmark tox
	cd examples/word-count && python setup.py install && pytest -v tests
	cd examples/rustapi_module && tox
