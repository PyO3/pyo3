.PHONY: test test_py publish clippy lint fmt

test: lint test_py
	cargo test

test_py:
	for example in examples/*; do tox -e py -c $$example || exit 1; done

fmt:
	cargo fmt --all -- --check
	black . --check

clippy:
	@touch src/lib.rs  # Touching file to ensure that cargo clippy will re-check the project
	cargo clippy --features="num-bigint num-complex hashbrown serde" --tests -- -Dwarnings
	cargo clippy --features="abi3 num-bigint num-complex hashbrown serde" --tests -- -Dwarnings
	for example in examples/*; do cargo clippy --manifest-path $$example/Cargo.toml -- -Dwarnings || exit 1; done

lint: fmt clippy
	@true

publish: test
	cargo publish --manifest-path pyo3-macros-backend/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish --manifest-path pyo3-macros/Cargo.toml
	sleep 10  # wait for crates.io to update
	cargo publish
