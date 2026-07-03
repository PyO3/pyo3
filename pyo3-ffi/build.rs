use pyo3_build_config::{
    bail, ensure,
    pyo3_build_script_impl::{
        cargo_env_var, env_var, errors::Result, is_linking_libpython_for_target,
        print_feature_cfgs, print_libpython_rpath_link_args, resolve_build_config,
        target_triple_from_env, BuildConfig, BuildConfigSource, MaximumVersionExceeded,
    },
    warn, InterpreterConfig, PythonAbiKind, PythonImplementation, PythonVersion, StableAbi,
};

/// Minimum Python version PyO3 supports.
struct SupportedVersions {
    min: PythonVersion,
    max: PythonVersion,
}

const SUPPORTED_VERSIONS_CPYTHON: SupportedVersions = SupportedVersions {
    min: PythonVersion { major: 3, minor: 9 },
    max: PythonVersion {
        major: 3,
        minor: 15,
    },
};

const SUPPORTED_VERSIONS_PYPY: SupportedVersions = SupportedVersions {
    min: PythonVersion {
        major: 3,
        minor: 11,
    },
    max: SUPPORTED_VERSIONS_CPYTHON.max,
};

const SUPPORTED_VERSIONS_GRAALPY: SupportedVersions = SupportedVersions {
    min: PythonVersion {
        major: 3,
        minor: 10,
    },
    max: SUPPORTED_VERSIONS_CPYTHON.max,
};

const MIN_FREE_THREADED_VERSION: PythonVersion = PythonVersion {
    major: 3,
    minor: 14,
};

const PY_3_15: PythonVersion = PythonVersion {
    major: 3,
    minor: 15,
};

fn ensure_python_version(interpreter_config: &InterpreterConfig) -> Result<()> {
    // This is an undocumented env var which is only really intended to be used in CI / for testing
    // and development.
    if std::env::var("UNSAFE_PYO3_SKIP_VERSION_CHECK").as_deref() == Ok("1") {
        return Ok(());
    }

    match interpreter_config.target_abi().implementation() {
        PythonImplementation::CPython => {
            let versions = SUPPORTED_VERSIONS_CPYTHON;
            let interp_version = interpreter_config.target_abi().version();
            ensure!(
                interpreter_config.version() >= versions.min,
                "the configured Python interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.version(),
                versions.min,
            );
            let v_plus_1 = PythonVersion {
                major: versions.max.major,
                minor: versions.max.minor + 1,
            };
            if interp_version == v_plus_1 {
                warn!(
                    "Using experimental support for the Python {}.{} ABI. \
                     Build artifacts may not be compatible with the final release of CPython, \
                     so do not distribute them.",
                    v_plus_1.major, v_plus_1.minor,
                );
            } else if interp_version > v_plus_1 {
                let mut error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                if interpreter_config.target_abi().kind().is_free_threaded() {
                    if interp_version >= PY_3_15 {
                        if env_var("PYO3_USE_STABLE_ABI_FORWARD_COMPATIBILITY")
                            .is_none_or(|os_str| os_str != "1")
                        {
                            error.add_help(
                                "set PYO3_USE_STABLE_ABI_FORWARD_COMPATIBILITY=1 to suppress this check and build anyway using the free-threaded stable ABI"
                            );
                            return Err(error.finish().into());
                        }
                    } else {
                        error.add_help(format!(
                            "the free-threaded build of CPython {}.{} does not support the limited API so this check cannot be suppressed.", interp_version.major, interp_version.minor
                        ).as_str());
                        return Err(error.finish().into());
                    }
                }
                if env_var("PYO3_USE_ABI3_FORWARD_COMPATIBILITY").is_none_or(|os_str| os_str != "1")
                    && env_var("PYO3_USE_STABLE_ABI_FORWARD_COMPATIBILITY")
                        .is_none_or(|os_str| os_str != "1")
                {
                    error.add_help("set PYO3_USE_STABLE_ABI_FORWARD_COMPATIBILITY=1 to suppress this check and build anyway using the stable ABI");
                    return Err(error.finish().into());
                }
            }

            if interpreter_config.target_abi().kind().is_free_threaded() {
                ensure!(
                    interpreter_config.target_abi().version() >= MIN_FREE_THREADED_VERSION,
                    "PyO3 does not support the free-threaded build of CPython versions below {}, the selected Python version is {}",
                    MIN_FREE_THREADED_VERSION,
                    interpreter_config.target_abi().version(),
                );
            }
        }
        PythonImplementation::PyPy => {
            let versions = SUPPORTED_VERSIONS_PYPY;
            ensure!(
                interpreter_config.target_abi().version() >= versions.min,
                "the configured PyPy interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.target_abi().version(),
                versions.min,
            );
            // PyO3 does not support abi3, so we cannot offer forward compatibility
            if interpreter_config.target_abi().version() > versions.max {
                let error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                return Err(error.finish().into());
            }
        }
        PythonImplementation::GraalPy => {
            let versions = SUPPORTED_VERSIONS_GRAALPY;
            ensure!(
                interpreter_config.target_abi().version() >= versions.min,
                "the configured GraalPy interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.target_abi().version(),
                versions.min,
            );
            // GraalPy does not support abi3, so we cannot offer forward compatibility
            if interpreter_config.target_abi().version() > versions.max {
                let error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                return Err(error.finish().into());
            }
        }
        PythonImplementation::RustPython => {}
    }

    if let PythonAbiKind::Stable(abi) = interpreter_config.target_abi().kind() {
        match interpreter_config.target_abi().implementation() {
            PythonImplementation::CPython => match abi {
                StableAbi::Abi3t => {
                    ensure!(
                        interpreter_config.target_abi().version() >= PY_3_15,
                        "Abi3t builds are not supported on CPython targets before Python 3.15"
                    )
                }
                StableAbi::Abi3 => {}
            },
            PythonImplementation::PyPy => warn!(
                "PyPy does not yet support {abi} so the build artifacts will be version-specific. \
                 See https://github.com/pypy/pypy/issues/3397 for more information."
            ),
            PythonImplementation::GraalPy => warn!(
                "GraalPy does not support {abi} so the build artifacts will be version-specific."
            ),
            PythonImplementation::RustPython => {}
        }
    }

    Ok(())
}

fn ensure_target_pointer_width(interpreter_config: &InterpreterConfig) -> Result<()> {
    if let Some(pointer_width) = interpreter_config.pointer_width() {
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

fn emit_link_config(build_config: &BuildConfig) -> Result<()> {
    let interpreter_config = &build_config.interpreter_config;
    let target_os = cargo_env_var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = cargo_env_var("CARGO_CFG_TARGET_ENV");
    let target_arch = cargo_env_var("CARGO_CFG_TARGET_ARCH");

    let lib_name = interpreter_config
        .lib_name()
        .ok_or("attempted to link to Python shared library but config does not contain lib_name")?;

    if target_os == "windows" {
        // Use raw-dylib linking: emit a cfg so that `extern_libpython!` picks the
        // right `#[link(name = "...", kind = "raw-dylib")]` attribute at compile time.
        // This eliminates the need for import libraries (.lib files) entirely.
        //
        // Note: raw-dylib is inherently dynamic linking. Static embedding of the
        // Python interpreter on Windows is not supported by this path (and is not
        // officially supported by CPython on Windows).
        println!("cargo:rustc-cfg=pyo3_dll=\"{lib_name}\"");

        let is_i686_pc_windows_gnu = matches!(target_env.as_deref(), Some("gnu"))
            && matches!(target_arch.as_deref(), Some("x86"));

        if is_i686_pc_windows_gnu {
            let import_lib_name = lib_name.strip_prefix("lib").unwrap_or(lib_name);
            println!("cargo:rustc-link-lib={import_lib_name}");

            if let Some(lib_dir) = interpreter_config.lib_dir() {
                println!("cargo:rustc-link-search=native={lib_dir}");
            } else if matches!(build_config.source, BuildConfigSource::CrossCompile) {
                warn!(
                    "The output binary will link to libpython, \
                    but PYO3_CROSS_LIB_DIR environment variable is not set. \
                    Ensure that the target Python library directory is \
                    in the rustc native library search path."
                );
            }
        }
    } else {
        println!(
            "cargo:rustc-link-lib={link_model}{lib_name}",
            link_model = if interpreter_config.shared() {
                ""
            } else {
                "static="
            },
        );

        if let Some(lib_dir) = interpreter_config.lib_dir() {
            println!("cargo:rustc-link-search=native={lib_dir}");
        } else if matches!(build_config.source, BuildConfigSource::CrossCompile) {
            warn!(
                "The output binary will link to libpython, \
                but PYO3_CROSS_LIB_DIR environment variable is not set. \
                Ensure that the target Python library directory is \
                in the rustc native library search path."
            );
        }
    }

    Ok(())
}

/// Prepares the PyO3 crate for compilation.
///
/// This uses pyo3-build-config implementation to detect the target Python interpreter and validate
/// it's suitable for building with.
///
/// Emits the cargo configuration based on this config as well as a few checks of the Rust compiler
/// version to enable features which aren't supported on MSRV.
fn configure_pyo3_ffi() -> Result<()> {
    let target = target_triple_from_env();
    let build_config = resolve_build_config(&target)?;
    let interpreter_config = &build_config.interpreter_config;

    if env_var("PYO3_PRINT_CONFIG").is_some_and(|os_str| os_str == "1") {
        print_config_and_exit(interpreter_config);
    }

    ensure_python_version(interpreter_config)?;
    ensure_target_pointer_width(interpreter_config)?;

    // Serialize the whole interpreter config into DEP_PYTHON_PYO3_CONFIG env var.
    interpreter_config.to_cargo_dep_env()?;

    if is_linking_libpython_for_target(&target)
        && !interpreter_config.suppress_build_script_link_lines()
    {
        emit_link_config(&build_config)?;
    }

    for cfg in interpreter_config.build_script_outputs() {
        println!("{cfg}")
    }

    // Extra lines come last, to support last write wins.
    for line in interpreter_config.extra_build_script_lines() {
        println!("{line}");
    }

    print_feature_cfgs();

    // Make `cargo test` etc work with non-system Python installations
    print_libpython_rpath_link_args(&target, interpreter_config);

    Ok(())
}

fn print_config_and_exit(config: &InterpreterConfig) {
    println!("\n-- PYO3_PRINT_CONFIG=1 is set, printing configuration and halting compile --");
    config
        .to_writer(std::io::stdout())
        .expect("failed to print config to stdout");
    println!("\nnote: unset the PYO3_PRINT_CONFIG environment variable and retry to compile with the above config");
    std::process::exit(101);
}

fn main() {
    pyo3_build_config::print_expected_cfgs();
    if let Err(e) = configure_pyo3_ffi() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
