.PHONY: test test_py publish clippy lint fmt

ALL_ADDITIVE_FEATURES = macros multiple-pymethods num-bigint num-complex hashbrown serde indexmap eyre anyhow

list_all_additive_features:
	@echo $(ALL_ADDITIVE_FEATURES)

test: lint test_py
	cargo test
	cargo test --features="abi3"
	cargo test --features="$(ALL_ADDITIVE_FEATURES)"
	cargo test --features="abi3 $(ALL_ADDITIVE_FEATURES)"

test_py:
	for example in examples/*/; do TOX_TESTENV_PASSENV=RUSTUP_HOME tox -e py -c $$example || exit 1; done

fmt:
	cargo fmt --all -- --check
	black . --check

clippy:
	cargo clippy --features="$(ALL_ADDITIVE_FEATURES)" --tests -- -Dwarnings
	cargo clippy --features="abi3 $(ALL_ADDITIVE_FEATURES)" --tests -- -Dwarnings
	for example in examples/*/; do cargo clippy --manifest-path $$example/Cargo.toml -- -Dwarnings || exit 1; done

lint: fmt clippy
	@true

publish: test
	cargo publish --manifest-path pyo3-macros-backend/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish --manifest-path pyo3-macros/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish
