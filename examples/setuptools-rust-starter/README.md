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

Alternatively, install tox and run the tests inside an isolated environment:

```shell
tox -e py
```
