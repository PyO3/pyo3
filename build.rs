use std::{env, process::Command};

use pyo3_build_config::pyo3_build_script_impl::{cargo_env_var, errors::Result};
use pyo3_build_config::{bail, InterpreterConfig};

fn ensure_auto_initialize_ok(interpreter_config: &InterpreterConfig) -> Result<()> {
    if cargo_env_var("CARGO_FEATURE_AUTO_INITIALIZE").is_some() {
        if !interpreter_config.shared {
            bail!(
                "The `auto-initialize` feature is enabled, but your python installation only supports \
                embedding the Python interpreter statically. If you are attempting to run tests, or a \
                binary which is okay to link dynamically, install a Python distribution which ships \
                with the Python shared library.\n\
                \n\
                Embedding the Python interpreter statically does not yet have first-class support in \
                PyO3. If you are sure you intend to do this, disable the `auto-initialize` feature.\n\
                \n\
                For more information, see \
                https://pyo3.rs/v{pyo3_version}/\
                    building_and_distribution.html#embedding-python-in-rust",
                pyo3_version = env::var("CARGO_PKG_VERSION").unwrap()
            );
        }

        // TODO: PYO3_CI env is a hack to workaround CI with PyPy, where the `dev-dependencies`
        // currently cause `auto-initialize` to be enabled in CI.
        // Once MSRV is 1.51 or higher, use cargo's `resolver = "2"` instead.
        if interpreter_config.implementation.is_pypy() && env::var_os("PYO3_CI").is_none() {
            bail!("the `auto-initialize` feature is not supported with PyPy");
        }
    }
    Ok(())
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = core::str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    pieces.next()?.parse().ok()
}

/// Prepares the PyO3 crate for compilation.
///
/// This loads the config from pyo3-build-config and then makes some additional checks to improve UX
/// for users.
///
/// Emits the cargo configuration based on this config as well as a few checks of the Rust compiler
/// version to enable features which aren't supported on MSRV.
fn configure_pyo3() -> Result<()> {
    let interpreter_config = pyo3_build_config::get();

    interpreter_config.emit_pyo3_cfgs();

    let rustc_minor_version = rustc_minor_version().unwrap_or(0);
    ensure_auto_initialize_ok(interpreter_config)?;

    // Enable use of const generics on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=min_const_generics");
    }

    // Enable use of std::ptr::addr_of! on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=addr_of");
    }

    Ok(())
}

fn main() {
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
