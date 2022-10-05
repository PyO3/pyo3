use std::env;

use pyo3_build_config::pyo3_build_script_impl::{cargo_env_var, errors::Result};
use pyo3_build_config::{bail, print_feature_cfgs, InterpreterConfig};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str;

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

    ensure_auto_initialize_ok(interpreter_config)?;

    // Emit cfgs like `addr_of` and `min_const_generics`
    print_feature_cfgs();

    Ok(())
}

// Stolen from https://github.com/dtolnay/thiserror/blob/master/build.rs
const PROBE: &str = r#"
    #![feature(rustc_attrs, negative_impls)]

    #[rustc_on_unimplemented(
        message="message",
        label="label",
        note="note"
    )]
    pub trait Foo {}

    pub struct Bar;

    impl !Send for Bar {}
"#;

fn compile_probe() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, PROBE).ok()?;

    // Make sure to pick up Cargo rustc configuration.
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER") {
        let mut cmd = Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    cmd.stderr(Stdio::null())
        .arg("--edition=2018")
        .arg("--crate-name=pyo3_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    cmd.status().ok()
}

fn main() {
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
    match compile_probe() {
        Some(status) if status.success() => println!("cargo:rustc-cfg=better_errors"),
        _ => {}
    }
}
