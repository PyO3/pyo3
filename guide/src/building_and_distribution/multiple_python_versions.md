# Supporting multiple Python versions

PyO3 supports all actively-supported Python 3 and PyPy versions. As much as possible, this is done internally to PyO3 so that your crate's code does not need to adapt to the differences between each version. However, as Python features grow and change between versions, PyO3 cannot a completely identical API for every Python version. This may require you to add conditional compilation to your crate or runtime checks for the Python version.

This section of the guide first introduces the `pyo3-build-config` crate, which you can use as a `build-dependency` to add additional `#[cfg]` flags which allow you to support multiple Python versions at compile-time.

Second, we'll show how to check the Python version at runtime. This can be useful when building for multiple versions with the `abi3` feature, where the Python API compiled against is not always the same as the one in use.

## Conditional compilation for different Python versions

The `pyo3-build-config` exposes multiple [`#[cfg]` flags](https://doc.rust-lang.org/rust-by-example/attribute/cfg.html) which can be used to conditionally compile code for a given Python version. PyO3 itself depends on this crate, so by using it you can be sure that you are configured correctly for the Python version PyO3 is building against.

This allows us to write code like the following

```rust,ignore
#[cfg(Py_3_7)]
fn function_only_supported_on_python_3_7_and_up() { }

#[cfg(not(Py_3_8))]
fn function_only_supported_before_python_3_8() { }

#[cfg(not(Py_LIMITED_API))]
fn function_incompatible_with_abi3_feature() { }
```

The following sections first show how to add these `#[cfg]` flags to your build process, and then cover some common patterns flags in a little more detail.

To see a full reference of all the `#[cfg]` flags provided, see the [`pyo3-build-cfg` docs](https://docs.rs/pyo3-build-config).

### Using `pyo3-build-config`

You can use the `#[cfg]` flags in just two steps:

1. Add `pyo3-build-config` it to your crate's build dependencies in `Cargo.toml`:

   ```toml
   [build-dependencies]
   pyo3-build-config = "{{#PYO3_CRATE_VERSION}}"
   ```

2. Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html) file to your crate with the following contents:

   ```rust,ignore
   fn main() {
       // If you have an existing build.rs file, just add this line to it.
       pyo3_build_config::use_pyo3_cfgs();
   }
   ```

After these steps you are ready to annotate your code!

### Common usages of `pyo3-build-cfg` flags

The following are some common patterns implemented using these flags:

// TODO

## Checking the Python version at runtime

// TODO
