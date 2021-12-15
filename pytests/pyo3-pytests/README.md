# pyo3-pytests

An extension module built using PyO3, used to test PyO3 from Python.

## Testing

This package is intended to be built using `maturin`. Once built, you can run the tests using `pytest`:

```shell
pip install maturin
maturin develop
pytest
```

Alternatively, install tox and run the tests inside an isolated environment:

```shell
tox -e py
```
