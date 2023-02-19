# PyO3 Examples

These example crates are a collection of toy native modules built with PyO3. They are all tested using `nox` in PyO3's CI.

Below is a brief description of each of these:

| Example | Description |
| ------- | ----------- |
| `decorator` | A project showcasing the example from the [Emulating callable objects](https://pyo3.rs/latest/class/call.html) chapter of the guide. |
| `maturin-starter` | A template project which is configured to use [`maturin`](https://github.com/PyO3/maturin) for development. |
| `setuptools-rust-starter` | A template project which is configured to use [`setuptools_rust`](https://github.com/PyO3/setuptools-rust/) for development. |
| `word-count` | A quick performance comparison between word counter implementations written in each of Rust and Python. |
| `plugin` | Illustrates how to use Python as a scripting language within a Rust application |

## Creating new projects from these examples

To copy an example, use [`cargo-generate`](https://crates.io/crates/cargo-generate). Follow the commands below, replacing `<example>` with the example to start from:

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/<example>
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
