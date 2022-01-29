# pyo3-benchmarks

This extension module contains benchmarks for pieces of PyO3's API accessible from Python.

## Running the benchmarks

You can install the module in your Python environment and then run the benchmarks with pytest:

```shell
pip install .
pytest --benchmark-enable
```

Or with nox:

```shell
nox -s bench
```
