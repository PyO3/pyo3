# pyo3-pytests

An extension module built using PyO3, used to test and benchmark PyO3 from Python.

The `stubs` directory contains Python stubs used to test the automated stubs introspection.
To test them run `nox -s test-introspection`.

## Testing

This package is intended to be built using `maturin`. Once built, you can run the tests using `pytest`:

```shell
pip install maturin
maturin develop
pytest
```

Alternatively, install nox and run the tests inside an isolated environment:

```shell
nox
```

## Running benchmarks

You can install the module in your Python environment and then run the benchmarks with pytest:

```shell
pip install .
pytest --benchmark-enable
```

Or with nox:

```shell
nox -s bench
```
