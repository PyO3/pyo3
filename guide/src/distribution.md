# Distribution

## `setuptools-rust` integration

[`setuptools-rust`][setuptools-rust] is a setuptools helpers for Rust Python extensions. It supports `PyO3` by default.

For detailed usage, please refer to its [README](https://github.com/PyO3/setuptools-rust/blob/master/README.rst)

## Source distribution

To build a source code distribution, you need to add the following lines to your `MANIFEST.in` file to ensure it correctly packages Rust extension source code.

```text
include Cargo.toml
recursive-include src *
```

Then you can build a source code distribution by (assuming you have already written a `setup.py`):

```bash
python setup.py sdist
```

## Binary wheel distribution

To build a binary wheel, [manylinux](https://github.com/pypa/manylinux) would be a natural choice for Linux.

Take the [example](https://github.com/PyO3/setuptools-rust/tree/master/example) project in [`setuptools-rust`][setuptools-rust] repository for example,
we have a [`build-wheels.sh`](https://github.com/PyO3/setuptools-rust/blob/master/example/build-wheels.sh) to be used with Docker to build manylinux1 wheels.
First you need to pull the `manylinux1` Docker image:

```bash
$ docker pull quay.io/pypa/manylinux1_x86_64
```

Then use the following command to build wheels for supported Python versions:

```bash
$ docker run --rm -v `pwd`:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
```

You will find all the wheels in `dist` directory:

```bash
$ ls dist
hello_rust-1.0-cp27-cp27m-linux_x86_64.whl       hello_rust-1.0-cp35-cp35m-linux_x86_64.whl
hello_rust-1.0-cp27-cp27m-manylinux1_x86_64.whl  hello_rust-1.0-cp35-cp35m-manylinux1_x86_64.whl
hello_rust-1.0-cp27-cp27mu-linux_x86_64.whl      hello_rust-1.0-cp36-cp36m-linux_x86_64.whl
hello_rust-1.0-cp27-cp27mu-manylinux1_x86_64.whl hello_rust-1.0-cp36-cp36m-manylinux1_x86_64.whl
```

The `*-manylinux1_x86_64.whl` files are the `manylinux1` wheels that you can upload to PyPI.

[setuptools-rust]: https://github.com/PyO3/setuptools-rust
