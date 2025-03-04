# `pyo3-ffi` Examples

These example crates are a collection of toy extension modules built with
`pyo3-ffi`. They are all tested using `nox` in PyO3's CI.

Below is a brief description of each of these:

| Example | Description |
| `word-count` | Illustrates how to use pyo3-ffi to write a static rust extension |
| `sequential` | Illustrates how to use pyo3-ffi to write subinterpreter-safe modules using multi-phase module initialization |

## Creating new projects from these examples

To copy an example, use [`cargo-generate`](https://crates.io/crates/cargo-generate). Follow the commands below, replacing `<example>` with the example to start from:

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/<example>
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
