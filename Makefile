.PHONY: test test_py3 publish clippy lint fmt

# Constants used in clippy target
CLIPPY_LINTS_TO_DENY := warnings
CLIPPY_LINTS_TO_ALLOW := clippy::new_ret_no_self

test:
	cargo test
	${MAKE} clippy
	tox
	for example in examples/*; do tox -e py -c $$example/tox.ini; done

test_py3:
	tox -e py3
	for example in examples/*; do tox -e py3 -c $$example/tox.ini; done

fmt:
	cargo fmt --all -- --check

clippy:
	@touch src/lib.rs  # Touching file to ensure that cargo clippy will re-check the project
	cargo clippy --all-features --all-targets -- \
		$(addprefix -D ,${CLIPPY_LINTS_TO_DENY}) \
		$(addprefix -A ,${CLIPPY_LINTS_TO_ALLOW})

lint: fmt clippy
	@true

publish: test
	cargo publish --manifest-path pyo3-derive-backend/Cargo.toml
	cargo publish --manifest-path pyo3cls/Cargo.toml
	cargo publish
