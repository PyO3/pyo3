use std::{env, process::Command};

use pyo3_build_config::{InterpreterConfig, PythonImplementation};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
                None => {
                    return Err("failed to configure `ld_version` when compiling for unix".into())
                }
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
            println!("{}", get_rustc_link_lib(&interpreter_config)?);
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
            println!("{}", get_rustc_link_lib(&interpreter_config)?);
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
            return Err(format!(
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
            )
            .into());
        }

        // TODO: PYO3_CI env is a hack to workaround CI with PyPy, where the `dev-dependencies`
        // currently cause `auto-initialize` to be enabled in CI.
        // Once cargo's `resolver = "2"` is stable (~ MSRV Rust 1.52), remove this.
        if interpreter_config.is_pypy() && env::var_os("PYO3_CI").is_none() {
            return Err("The `auto-initialize` feature is not supported with PyPy.".into());
        }
    }

    Ok(())
}

fn configure_pyo3() -> Result<()> {
    let cfg = pyo3_build_config::get();
    emit_cargo_configuration(&cfg)?;
    cfg.emit_pyo3_cfgs();

    // Enable use of const generics on Rust 1.51 and greater
    if rustc_minor_version().unwrap_or(0) >= 51 {
        println!("cargo:rustc-cfg=min_const_generics");
    }

    Ok(())
}

fn main() {
    // Print out error messages using display, to get nicer formatting.
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e);
        std::process::exit(1)
    }
}
