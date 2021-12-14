.PHONY: test test_py publish clippy lint fmt fmt_py fmt_rust

ALL_ADDITIVE_FEATURES = macros multiple-pymethods num-bigint num-complex hashbrown serde indexmap eyre anyhow

list_all_additive_features:
	@echo $(ALL_ADDITIVE_FEATURES)

test: lint test_py
	cargo test
	cargo test --features="abi3"
	cargo test --features="$(ALL_ADDITIVE_FEATURES)"
	cargo test --features="abi3 $(ALL_ADDITIVE_FEATURES)"

test_py:
	@for example in examples/*/; do echo "-- Testing $$example --"; TOX_TESTENV_PASSENV=RUSTUP_HOME tox -e py -c $$example || exit 1; echo ""; done
	@for package in pytests/*/; do echo "-- Testing $$package --"; TOX_TESTENV_PASSENV=RUSTUP_HOME tox -e py -c $$package || exit 1; echo ""; done

fmt_py:
	black . --check

fmt_rust:
	cargo fmt --all -- --check
	for package in pytests/*/; do cargo fmt --manifest-path $$package/Cargo.toml -- --check || exit 1; done

fmt: fmt_rust fmt_py
	@true

clippy:
	cargo clippy --features="$(ALL_ADDITIVE_FEATURES)" --all-targets --workspace -- -Dwarnings
	cargo clippy --features="abi3 $(ALL_ADDITIVE_FEATURES)" --all-targets --workspace -- -Dwarnings
	for example in examples/*/; do cargo clippy --manifest-path $$example/Cargo.toml -- -Dwarnings || exit 1; done
	for package in pytests/*/; do cargo clippy --manifest-path $$package/Cargo.toml -- -Dwarnings || exit 1; done

lint: fmt clippy
	@true

publish: test
	cargo publish --manifest-path pyo3-build-config/Cargo.toml
	sleep 10
	cargo publish --manifest-path pyo3-macros-backend/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish --manifest-path pyo3-macros/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish
