.PHONY: test publish

test:
	cargo test
	cargo clippy
	tox
	for example in examples/*; do tox -e py --workdir $$example; done

publish:
	cargo test
	cargo publish --manifest-path pyo3-derive-backend/Cargo.toml
	cargo publish --manifest-path pyo3cls/Cargo.toml
	cargo publish
