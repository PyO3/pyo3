use pyo3_build_config::{
    bail, ensure, print_feature_cfgs,
    pyo3_build_script_impl::{
        errors::Result, is_linking_libpython_for_target, resolve_build_config,
        target_triple_from_env, BuildConfig, BuildConfigSource, InterpreterConfig,
        MaximumVersionExceeded, PythonVersion,
    },
    warn, PythonImplementation, BUILD_CTX,
};

/// Minimum Python version PyO3 supports.
struct SupportedVersions {
    min: PythonVersion,
    max: PythonVersion,
}

const SUPPORTED_VERSIONS_CPYTHON: SupportedVersions = SupportedVersions {
    min: PythonVersion { major: 3, minor: 8 },
    max: PythonVersion {
        major: 3,
        minor: 14,
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

fn ensure_python_version(interpreter_config: &InterpreterConfig) -> Result<()> {
    // This is an undocumented env var which is only really intended to be used in CI / for testing
    // and development.
    if std::env::var("UNSAFE_PYO3_SKIP_VERSION_CHECK").as_deref() == Ok("1") {
        return Ok(());
    }

    match interpreter_config.implementation {
        PythonImplementation::CPython => {
            let versions = SUPPORTED_VERSIONS_CPYTHON;
            ensure!(
                interpreter_config.version >= versions.min,
                "the configured Python interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.version,
                versions.min,
            );
            let v_plus_1 = PythonVersion {
                major: versions.max.major,
                minor: versions.max.minor + 1,
            };
            if interpreter_config.version == v_plus_1 {
                warn!(
                    "Using experimental support for the Python {}.{} ABI. \
                     Build artifacts may not be compatible with the final release of CPython, \
                     so do not distribute them.",
                    v_plus_1.major, v_plus_1.minor,
                );
            } else if interpreter_config.version > v_plus_1 {
                let mut error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                if interpreter_config.is_free_threaded() {
                    error.add_help(
                        "the free-threaded build of CPython does not support the limited API so this check cannot be suppressed.",
                    );
                    return Err(error.finish().into());
                }

                if !*BUILD_CTX.ext.use_abi13_forward_compatibility {
                    error.add_help("set PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 to suppress this check and build anyway using the stable ABI");
                    return Err(error.finish().into());
                }
            }

            if interpreter_config.is_free_threaded() {
                let min_free_threaded_version = PythonVersion {
                    major: 3,
                    minor: 14,
                };
                ensure!(
                    interpreter_config.version >= min_free_threaded_version,
                    "PyO3 does not support the free-threaded build of CPython versions below {}, the selected Python version is {}",
                    min_free_threaded_version,
                    interpreter_config.version,
                );
            }
        }
        PythonImplementation::PyPy => {
            let versions = SUPPORTED_VERSIONS_PYPY;
            ensure!(
                interpreter_config.version >= versions.min,
                "the configured PyPy interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.version,
                versions.min,
            );
            // PyO3 does not support abi3, so we cannot offer forward compatibility
            if interpreter_config.version > versions.max {
                let error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                return Err(error.finish().into());
            }
        }
        PythonImplementation::GraalPy => {
            let versions = SUPPORTED_VERSIONS_GRAALPY;
            ensure!(
                interpreter_config.version >= versions.min,
                "the configured GraalPy interpreter version ({}) is lower than PyO3's minimum supported version ({})",
                interpreter_config.version,
                versions.min,
            );
            // GraalPy does not support abi3, so we cannot offer forward compatibility
            if interpreter_config.version > versions.max {
                let error = MaximumVersionExceeded::new(interpreter_config, versions.max);
                return Err(error.finish().into());
            }
        }
        PythonImplementation::RustPython => {}
    }

    if interpreter_config.abi3 {
        match interpreter_config.implementation {
            PythonImplementation::CPython => {
                if interpreter_config.is_free_threaded() {
                    warn!(
                            "The free-threaded build of CPython does not yet support abi3 so the build artifacts will be version-specific."
                    )
                }
            }
            PythonImplementation::PyPy => warn!(
                "PyPy does not yet support abi3 so the build artifacts will be version-specific. \
                See https://github.com/pypy/pypy/issues/3397 for more information."
            ),
            PythonImplementation::GraalPy => warn!(
                "GraalPy does not support abi3 so the build artifacts will be version-specific."
            ),
            PythonImplementation::RustPython => {}
        }
    }

    Ok(())
}

fn ensure_target_pointer_width(interpreter_config: &InterpreterConfig) -> Result<()> {
    if let Some(pointer_width) = interpreter_config.pointer_width {
        // Try to check whether the target architecture matches the python library
        let rust_target = match &*BUILD_CTX.cargo.cargo_cfg_target_pointer_width {
            Ok(target) => *target,
            Err(e) => bail!("{e}"),
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
    let target_os = BUILD_CTX.cargo.cargo_cfg_target_os.clone();

    let lib_name = interpreter_config
        .lib_name
        .as_ref()
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
    } else {
        println!(
            "cargo:rustc-link-lib={link_model}{lib_name}",
            link_model = if interpreter_config.shared {
                ""
            } else {
                "static="
            },
        );

        if let Some(lib_dir) = &interpreter_config.lib_dir {
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
/// This loads the config from pyo3-build-config and then makes some additional checks to improve UX
/// for users.
///
/// Emits the cargo configuration based on this config as well as a few checks of the Rust compiler
/// version to enable features which aren't supported on MSRV.
fn configure_pyo3() -> Result<()> {
    let target = target_triple_from_env();
    let build_config = resolve_build_config(&target)?;
    let interpreter_config = &build_config.interpreter_config;

    if *BUILD_CTX.ext.is_print_config {
        print_config_and_exit(interpreter_config);
    }

    ensure_python_version(interpreter_config)?;
    ensure_target_pointer_width(interpreter_config)?;

    // Serialize the whole interpreter config into DEP_PYTHON_PYO3_CONFIG env var.
    interpreter_config.to_cargo_dep_env()?;

    if is_linking_libpython_for_target(&target)
        && !interpreter_config.suppress_build_script_link_lines
    {
        emit_link_config(&build_config)?;
    }

    for cfg in interpreter_config.build_script_outputs() {
        println!("{cfg}")
    }

    // Extra lines come last, to support last write wins.
    for line in &interpreter_config.extra_build_script_lines {
        println!("{line}");
    }

    print_feature_cfgs();

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
    if let Err(e) = configure_pyo3() {
        eprintln!("error: {}", e.report());
        std::process::exit(1)
    }
}
