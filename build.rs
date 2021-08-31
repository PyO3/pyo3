use std::{env, process::Command};

use pyo3_build_config::{
    bail, ensure,
    pyo3_build_script_impl::{
        cargo_env_var, env_var, errors::Result, resolve_interpreter_config, InterpreterConfig,
        PythonVersion,
    },
};

/// Minimum Python version PyO3 supports.
const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 6 };

fn ensure_python_version(interpreter_config: &InterpreterConfig) -> Result<()> {
    ensure!(
        interpreter_config.version >= MINIMUM_SUPPORTED_VERSION,
        "the configured Python interpreter version ({}) is lower than PyO3's minimum supported version ({})",
        interpreter_config.version,
        MINIMUM_SUPPORTED_VERSION,
    );

    Ok(())
}

fn ensure_target_pointer_width(interpreter_config: &InterpreterConfig) -> Result<()> {
    if let Some(pointer_width) = interpreter_config.pointer_width {
        // Try to check whether the target architecture matches the python library
        let rust_target = match cargo_env_var("CARGO_CFG_TARGET_POINTER_WIDTH")
            .unwrap()
            .as_str()
        {
            "64" => 64,
            "32" => 32,
            x => bail!("unexpected Rust target pointer width: {}", x),
        };

        ensure!(
            rust_target == pointer_width,
            "your Rust target architecture ({}-bit) does not match your python interpreter ({}-bit)",
            rust_target,
            pointer_width
        );
    }
    Ok(())
}

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

fn emit_link_config(interpreter_config: &InterpreterConfig) -> Result<()> {
    let target_os = cargo_env_var("CARGO_CFG_TARGET_OS").unwrap();
    let is_extension_module = cargo_env_var("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    if target_os == "windows" || target_os == "android" || !is_extension_module {
        // windows and android - always link
        // other systems - only link if not extension module
        println!(
            "cargo:rustc-link-lib={link_model}{alias}{lib_name}",
            link_model = if interpreter_config.shared {
                ""
            } else {
                "static="
            },
            alias = if target_os == "windows" {
                "pythonXY:"
            } else {
                ""
            },
            lib_name = interpreter_config.lib_name.as_ref().ok_or(
                "attempted to link to Python shared library but config does not contain lib_name"
            )?,
        );
        if let Some(lib_dir) = &interpreter_config.lib_dir {
            println!("cargo:rustc-link-search=native={}", lib_dir);
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
    let interpreter_config = resolve_interpreter_config()?;

    if env_var("PYO3_PRINT_CONFIG").map_or(false, |os_str| os_str == "1") {
        print_config_and_exit(&interpreter_config);
    }

    ensure_python_version(&interpreter_config)?;
    ensure_target_pointer_width(&interpreter_config)?;
    ensure_auto_initialize_ok(&interpreter_config)?;

    if !interpreter_config.suppress_build_script_link_lines {
        emit_link_config(&interpreter_config)?;
    }

    interpreter_config.emit_pyo3_cfgs();

    let rustc_minor_version = rustc_minor_version().unwrap_or(0);

    // Enable use of #[track_caller] on Rust 1.46 and greater
    if rustc_minor_version >= 46 {
        println!("cargo:rustc-cfg=track_caller");
    }

    // Enable use of const generics on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=min_const_generics");
    }

    // Enable use of std::ptr::addr_of! on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=addr_of");
    }

    // Extra lines come last, to support last write wins.
    for line in &interpreter_config.extra_build_script_lines {
        println!("{}", line);
    }

    Ok(())
}

fn print_config_and_exit(config: &InterpreterConfig) {
    println!("\n-- PYO3_PRINT_CONFIG=1 is set, printing configuration and halting compile --");
    config
        .to_writer(&mut std::io::stdout())
        .expect("failed to print config to stdout");
    std::process::exit(101);
}

fn main() {
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
