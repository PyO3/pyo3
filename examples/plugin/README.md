# plugin

An example of a Rust app that uses Python for a plugin. A Python native module built using PyO3 and [`maturin`](https://github.com/PyO3/maturin) is used to provide
interface types that can be used to exchange data between Rust and Python. This also deals with how to separately test and load python modules.

# Building and Testing
## Host application
To run the app itself, you only need to run 

```shell
cargo run
```
It will build the app, as well as the plugin API, then run the app, load the plugin and show it working.

## Plugin API testing

The plugin API is in a separate crate `plugin_api`, so you can test it separately from the main app. 

To build the API only package, first install `maturin`:

```shell
pip install maturin
```

When building the plugin, simply using `maturin develop` will fail to produce a viable native module due to the features arrangement of PyO3. 
Instead, one needs to enable the optional feature as follows:

```shell
cd plugin_api
maturin build --features native
```

Alternatively, install nox and run the tests inside an isolated environment:

```shell
nox
```

## Copying this example

Use [`cargo-generate`](https://crates.io/crates/cargo-generate):

```bash
$ cargo install cargo-generate
$ cargo generate --git https://github.com/PyO3/pyo3 examples/plugin
```

(`cargo generate` will take a little while to clone the PyO3 repo first; be patient when waiting for the command to run.)
