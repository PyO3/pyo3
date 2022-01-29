# word-count

Demonstrates searching for a file in plain python, with rust singlethreaded and with rust multithreaded.

## Build

```shell
pip install .
```

## Usage

```python
from word_count import search_py, search, search_sequential

search_py("foo bar", "foo")
search("foo bar", "foo")
search_sequential("foo bar", "foo")
```

## Testing

To test install nox globally and run

```shell
nox
```

## Benchmark

To test install nox globally and run

```shell
nox -s bench
```

## Copying this example

Use [`cargo-generate`](https://crates.io/crates/cargo-generate):

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/word-count
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
