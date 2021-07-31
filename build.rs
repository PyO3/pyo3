use std::{env, process::Command};

use pyo3_build_config::{
    bail, ensure,
    errors::{Context, Result},
    InterpreterConfig, PythonImplementation, PythonVersion,
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

fn ensure_target_architecture(interpreter_config: &InterpreterConfig) -> Result<()> {
    // Try to check whether the target architecture matches the python library
    let rust_target = match env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap().as_str() {
        "64" => "64-bit",
        "32" => "32-bit",
        x => bail!("unexpected Rust target pointer width: {}", x),
    };

    // The reason we don't use platform.architecture() here is that it's not
    // reliable on macOS. See https://stackoverflow.com/a/1405971/823869.
    // Similarly, sys.maxsize is not reliable on Windows. See
    // https://stackoverflow.com/questions/1405913/how-do-i-determine-if-my-python-shell-is-executing-in-32bit-or-64bit-mode-on-os/1405971#comment6209952_1405971
    // and https://stackoverflow.com/a/3411134/823869.
    let python_target = match interpreter_config.calcsize_pointer {
        Some(8) => "64-bit",
        Some(4) => "32-bit",
        None => {
            // Unset, e.g. because we're cross-compiling. Don't check anything
            // in this case.
            return Ok(());
        }
        Some(n) => bail!("unexpected Python calcsize_pointer value: {}", n),
    };

    ensure!(
        rust_target == python_target,
        "Your Rust target architecture ({}) does not match your python interpreter ({})",
        rust_target,
        python_target
    );

    Ok(())
}

fn get_rustc_link_lib(config: &InterpreterConfig) -> Result<String> {
    let link_name = if env::var_os("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        if config.abi3 {
            // Link against python3.lib for the stable ABI on Windows.
            // See https://www.python.org/dev/peps/pep-0384/#linkage
            //
            // This contains only the limited ABI symbols.
            "pythonXY:python3".to_owned()
        } else if env::var_os("CARGO_CFG_TARGET_ENV").unwrap() == "gnu" {
            // https://packages.msys2.org/base/mingw-w64-python
            format!(
                "pythonXY:python{}.{}",
                config.version.major, config.version.minor
            )
        } else {
            format!(
                "pythonXY:python{}{}",
                config.version.major, config.version.minor
            )
        }
    } else {
        match config.implementation {
            PythonImplementation::CPython => match &config.ld_version {
                Some(ld_version) => format!("python{}", ld_version),
                None => bail!("failed to configure `ld_version` when compiling for unix"),
            },
            PythonImplementation::PyPy => format!("pypy{}-c", config.version.major),
        }
    };

    Ok(format!(
        "cargo:rustc-link-lib={link_model}{link_name}",
        link_model = if config.shared { "" } else { "static=" },
        link_name = link_name
    ))
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
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_extension_module = env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_some();
    match (is_extension_module, target_os.as_str()) {
        (_, "windows") => {
            // always link on windows, even with extension module
            println!("{}", get_rustc_link_lib(interpreter_config)?);
            // Set during cross-compiling.
            if let Some(libdir) = &interpreter_config.libdir {
                println!("cargo:rustc-link-search=native={}", libdir);
            }
            // Set if we have an interpreter to use.
            if let Some(base_prefix) = &interpreter_config.base_prefix {
                println!("cargo:rustc-link-search=native={}\\libs", base_prefix);
            }
        }
        (true, "macos") => {
            // with extension module on macos some extra linker arguments are needed
            println!("cargo:rustc-cdylib-link-arg=-undefined");
            println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
        }
        (false, _) | (_, "android") => {
            // other systems, only link libs if not extension module
            // android always link.
            println!("{}", get_rustc_link_lib(interpreter_config)?);
            if let Some(libdir) = &interpreter_config.libdir {
                println!("cargo:rustc-link-search=native={}", libdir);
            }
            if interpreter_config.implementation == PythonImplementation::PyPy {
                // PyPy 7.3.4 changed LIBDIR to point to base_prefix/lib as a regression, so need
                // to hard-code /bin search path too: https://foss.heptapod.net/pypy/pypy/-/issues/3442
                //
                // TODO: this workaround can probably be removed when PyPy 7.3.5 is released (and we
                // can call it a PyPy bug).
                if let Some(base_prefix) = &interpreter_config.base_prefix {
                    println!("cargo:rustc-link-search=native={}/bin", base_prefix);
                }
            }
        }
        _ => {}
    }

    if env::var_os("CARGO_FEATURE_AUTO_INITIALIZE").is_some() {
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
        // Once cargo's `resolver = "2"` is stable (~ MSRV Rust 1.52), remove this.
        if interpreter_config.is_pypy() && env::var_os("PYO3_CI").is_none() {
            bail!("The `auto-initialize` feature is not supported with PyPy.");
        }
    }

    Ok(())
}

/// Generates the interpreter config suitable for the host / target / cross-compilation at hand.
///
/// The result is written to pyo3_build_config::PATH, which downstream scripts can read from
/// (including `pyo3-macros-backend` during macro expansion).
fn configure_pyo3() -> Result<()> {
    let interpreter_config = pyo3_build_config::make_interpreter_config()?;
    ensure_python_version(&interpreter_config)?;
    ensure_target_architecture(&interpreter_config)?;
    emit_cargo_configuration(&interpreter_config)?;
    interpreter_config.to_writer(
        &mut std::fs::File::create(pyo3_build_config::PATH).with_context(|| {
            format!(
                "failed to create config file at {}",
                pyo3_build_config::PATH
            )
        })?,
    )?;
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
