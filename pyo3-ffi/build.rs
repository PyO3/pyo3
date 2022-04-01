use pyo3_build_config::{
    bail, ensure, print_feature_cfgs,
    pyo3_build_script_impl::{
        cargo_env_var, env_var, errors::Result, is_linking_libpython, resolve_interpreter_config,
        InterpreterConfig, PythonVersion,
    },
};

/// Minimum Python version PyO3 supports.
const MINIMUM_SUPPORTED_VERSION: PythonVersion = PythonVersion { major: 3, minor: 7 };

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

fn emit_link_config(interpreter_config: &InterpreterConfig) -> Result<()> {
    let target_os = cargo_env_var("CARGO_CFG_TARGET_OS").unwrap();

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

    // Serialize the whole interpreter config into DEP_PYTHON_PYO3_CONFIG env var.
    interpreter_config.to_cargo_dep_env()?;

    if is_linking_libpython() && !interpreter_config.suppress_build_script_link_lines {
        emit_link_config(&interpreter_config)?;
    }

    interpreter_config.emit_pyo3_cfgs();

    // Extra lines come last, to support last write wins.
    for line in &interpreter_config.extra_build_script_lines {
        println!("{}", line);
    }

    // Emit cfgs like `addr_of` and `min_const_generics`
    print_feature_cfgs();

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
