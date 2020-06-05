.PHONY: test test_py publish clippy lint fmt

# Constant used in clippy target
CLIPPY_LINTS_TO_DENY := warnings

test: lint test_py
	cargo test

test_py:
	tox -e py
	for example in examples/*; do tox -e py -c $$example/tox.ini || exit 1; done

fmt:
	cargo fmt --all -- --check
	black . --check

clippy:
	@touch src/lib.rs  # Touching file to ensure that cargo clippy will re-check the project
	cargo clippy --all-features --all-targets -- \
		$(addprefix -D ,${CLIPPY_LINTS_TO_DENY})

lint: fmt clippy
	@true

publish: test
	cargo publish --manifest-path pyo3-derive-backend/Cargo.toml
	cargo publish --manifest-path pyo3cls/Cargo.toml
	cargo publish
