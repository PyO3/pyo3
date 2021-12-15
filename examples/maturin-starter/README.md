# maturin-starter

An example of a basic Python extension module built using PyO3 and [`maturin`](https://github.com/PyO3/maturin).

## Building and Testing

To build this package, first install `maturin`:

```shell
pip install maturin
```

To build and test use `maturin develop`:

```shell
pip install -r requirements-dev.txt
maturin develop && pytest
```

Alternatively, install tox and run the tests inside an isolated environment:

```shell
tox -e py
```

## Copying this example

Use [`cargo-generate`](https://crates.io/crates/cargo-generate):

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/maturin-starter
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
