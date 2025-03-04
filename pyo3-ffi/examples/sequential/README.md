# sequential

A project built using only `pyo3_ffi`, without any of PyO3's safe api. It can be executed by subinterpreters that have their own GIL.

## Building and Testing

To build this package, first install `maturin`:

```shell
pip install maturin
```

To build and test use `maturin develop`:

```shell
pip install -r requirements-dev.txt
maturin develop
pytest
```

Alternatively, install nox and run the tests inside an isolated environment:

```shell
nox
```

## Copying this example

Use [`cargo-generate`](https://crates.io/crates/cargo-generate):

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/sequential
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
