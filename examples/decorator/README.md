# decorator

A project showcasing the example from the [Emulating callable objects](https://pyo3.rs/latest/class/call.html) chapter of the guide.

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

Alternatively, install tox and run the tests inside an isolated environment:

```shell
tox -e py
```
