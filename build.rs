use std::{env, ffi::OsString, path::Path, process::Command};

use pyo3_build_config::{
    bail, cargo_env_var, ensure, env_var,
    errors::{Context, Result},
    InterpreterConfig, PythonVersion,
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

fn ensure_target_pointer_width(pointer_width: u32) -> Result<()> {
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

fn emit_cargo_configuration(interpreter_config: &InterpreterConfig) -> Result<()> {
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
            lib_name = interpreter_config
                .lib_name
                .as_ref()
                .ok_or("config does not contain lib_name")?,
        );
        if let Some(lib_dir) = &interpreter_config.lib_dir {
            println!("cargo:rustc-link-search=native={}", lib_dir);
        }
    }

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
        if interpreter_config.is_pypy() && env::var_os("PYO3_CI").is_none() {
            bail!("the `auto-initialize` feature is not supported with PyPy");
        }
    }

    Ok(())
}

/// Generates the interpreter config suitable for the host / target / cross-compilation at hand.
///
/// The result is written to pyo3_build_config::PATH, which downstream scripts can read from
/// (including `pyo3-macros-backend` during macro expansion).
fn configure_pyo3() -> Result<()> {
    let write_config_file = env_var("PYO3_WRITE_CONFIG_FILE").map_or(false, |os_str| os_str == "1");
    let custom_config_file_path = env_var("PYO3_CONFIG_FILE");
    if let Some(path) = &custom_config_file_path {
        ensure!(
            Path::new(path).is_absolute(),
            "PYO3_CONFIG_FILE must be absolute"
        );
    }
    let (interpreter_config, path_to_write) = match (write_config_file, custom_config_file_path) {
        (true, Some(path)) => {
            // Create new interpreter config and write it to config file
            (pyo3_build_config::make_interpreter_config()?, Some(path))
        }
        (true, None) => bail!("PYO3_CONFIG_FILE must be set when PYO3_WRITE_CONFIG_FILE is set"),
        (false, Some(path)) => {
            // Read custom config file
            let path = Path::new(&path);
            println!("cargo:rerun-if-changed={}", path.display());
            let config_file = std::fs::File::open(path)
                .with_context(|| format!("failed to read config file at {}", path.display()))?;
            let reader = std::io::BufReader::new(config_file);
            (
                pyo3_build_config::InterpreterConfig::from_reader(reader)?,
                None,
            )
        }
        (false, None) => (
            // Create new interpreter config and write it to the default location
            pyo3_build_config::make_interpreter_config()?,
            Some(OsString::from(pyo3_build_config::DEFAULT_CONFIG_PATH)),
        ),
    };

    if let Some(path) = path_to_write {
        let path = Path::new(&path);
        let parent_dir = path.parent().ok_or_else(|| {
            format!(
                "failed to resolve parent directory of config file {}",
                path.display()
            )
        })?;
        std::fs::create_dir_all(&parent_dir).with_context(|| {
            format!(
                "failed to create config file directory {}",
                parent_dir.display()
            )
        })?;
        interpreter_config
            .to_writer(&mut std::fs::File::create(&path).with_context(|| {
                format!("failed to create config file at {}", path.display())
            })?)?;
    }
    if env_var("PYO3_PRINT_CONFIG").map_or(false, |os_str| os_str == "1") {
        print_config_and_exit(&interpreter_config);
    }

    ensure_python_version(&interpreter_config)?;
    if let Some(pointer_width) = interpreter_config.pointer_width {
        ensure_target_pointer_width(pointer_width)?;
    }
    emit_cargo_configuration(&interpreter_config)?;
    interpreter_config.emit_pyo3_cfgs();

    let rustc_minor_version = rustc_minor_version().unwrap_or(0);

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

fn print_config_and_exit(config: &InterpreterConfig) {
    println!("\n-- PYO3_PRINT_CONFIG=1 is set, printing configuration and halting compile --");
    config
        .to_writer(&mut std::io::stdout())
        .expect("failed to print config to stdout");
    std::process::exit(101);
}

fn main() {
    // Print out error messages using display, to get nicer formatting.
    if let Err(e) = configure_pyo3() {
        use std::error::Error;
        eprintln!("error: {}", e);
        let mut source = e.source();
        if source.is_some() {
            eprintln!("caused by:");
            let mut index = 0;
            while let Some(some_source) = source {
                eprintln!("  - {}: {}", index, some_source);
                source = some_source.source();
                index += 1;
            }
        }
        std::process::exit(1)
    }
}
