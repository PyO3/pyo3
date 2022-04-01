# setuptools-rust-starter

An example of a basic Python extension module built using PyO3 and [`setuptools_rust`](https://github.com/PyO3/setuptools-rust).

## Building and Testing

To build this package, first install `setuptools_rust`:

```shell
pip install setuptools_rust
```

To build and test use `python setup.py develop`:

```shell
pip install -r requirements-dev.txt
python setup.py develop && pytest
```

Alternatively, install nox and run the tests inside an isolated environment:

```shell
nox
```

## Copying this example

Use [`cargo-generate`](https://crates.io/crates/cargo-generate):

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/setuptools-rust-starter
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
