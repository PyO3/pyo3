//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! This crate exposes functionality to be called from build scripts to simplify building crates
//! which depend on PyO3.
//!
//! It used internally by the PyO3 crate's build script to apply the same configuration.

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]

mod errors;
mod impl_;

use std::{env, process::Command, str::FromStr, sync::OnceLock};

pub use impl_::{
    cross_compiling_from_to, find_all_sysconfigdata, parse_sysconfigdata, BuildFlag, BuildFlags,
    CrossCompileConfig, InterpreterConfig, PythonImplementation, PythonVersion, Triple,
};

use target_lexicon::OperatingSystem;

/// Adds all the [`#[cfg]` flags](index.html) to the current compilation.
///
/// This should be called from a build script.
///
/// The full list of attributes added are the following:
///
/// | Flag | Description |
/// | ---- | ----------- |
/// | `#[cfg(Py_3_7)]`, `#[cfg(Py_3_8)]`, `#[cfg(Py_3_9)]`, `#[cfg(Py_3_10)]` | These attributes mark code only for a given Python version and up. For example, `#[cfg(Py_3_7)]` marks code which can run on Python 3.7 **and newer**. |
/// | `#[cfg(Py_LIMITED_API)]` | This marks code which is run when compiling with PyO3's `abi3` feature enabled. |
/// | `#[cfg(Py_GIL_DISABLED)]` | This marks code which is run on the free-threaded interpreter. |
/// | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
/// | `#[cfg(GraalPy)]` | This marks code which is run when compiling for GraalPy. |
///
/// For examples of how to use these attributes,
#[doc = concat!("[see PyO3's guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/building-and-distribution/multiple_python_versions.html)")]
/// .
pub fn use_pyo3_cfgs() {
    print_expected_cfgs();
    for cargo_command in get().build_script_outputs() {
        println!("{cargo_command}")
    }
}

/// Adds linker arguments suitable for linking an extension module.
///
/// This should be called from a build script.
///
/// The following link flags are added:
/// - macOS: `-undefined dynamic_lookup`
/// - wasm32-unknown-emscripten: `-sSIDE_MODULE=2 -sWASM_BIGINT`
///
/// All other platforms currently are no-ops, however this may change as necessary
/// in future.
pub fn add_extension_module_link_args() {
    _add_extension_module_link_args(&impl_::target_triple_from_env(), std::io::stdout())
}

fn _add_extension_module_link_args(triple: &Triple, mut writer: impl std::io::Write) {
    if matches!(triple.operating_system, OperatingSystem::Darwin(_)) {
        writeln!(writer, "cargo:rustc-cdylib-link-arg=-undefined").unwrap();
        writeln!(writer, "cargo:rustc-cdylib-link-arg=dynamic_lookup").unwrap();
    } else if triple == &Triple::from_str("wasm32-unknown-emscripten").unwrap() {
        writeln!(writer, "cargo:rustc-cdylib-link-arg=-sSIDE_MODULE=2").unwrap();
        writeln!(writer, "cargo:rustc-cdylib-link-arg=-sWASM_BIGINT").unwrap();
    }
}

/// Adds linker arguments to set rpath when embedding Python within a Rust binary.
///
/// When running tests or binaries built with PyO3, the Python dynamic library needs
/// to be found at runtime.
///
/// This can be done by setting environment variables like `DYLD_LIBRARY_PATH` on macOS,
/// `LD_LIBRARY_PATH` on Linux, or `PATH` on Windows.
///
/// Altrnatively (as per this function) rpath can be set at link time to point to the
/// directory containing the Python dynamic library. This avoids the need to set environment
/// variables, so can be convenient, however may not be appropriate for binaries packaged
/// for distribution.
///
#[doc = concat!("[See PyO3's guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/building-and-distribution#dynamically-embedding-the-python-interpreter)")]
/// for more details.
pub fn add_libpython_rpath_link_args() {
    let target = impl_::target_triple_from_env();
    _add_libpython_rpath_link_args(
        get(),
        impl_::is_linking_libpython_for_target(&target),
        std::io::stdout(),
    )
}

fn _add_libpython_rpath_link_args(
    interpreter_config: &InterpreterConfig,
    is_linking_libpython: bool,
    mut writer: impl std::io::Write,
) {
    if is_linking_libpython {
        if let Some(lib_dir) = interpreter_config.lib_dir.as_ref() {
            writeln!(writer, "cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}").unwrap();
        }
    }
}

/// Adds linker arguments suitable for linking against the Python framework on macOS.
///
/// This should be called from a build script.
///
/// The following link flags are added:
/// - macOS: `-Wl,-rpath,<framework_prefix>`
///
/// All other platforms currently are no-ops.
pub fn add_python_framework_link_args() {
    let target = impl_::target_triple_from_env();
    _add_python_framework_link_args(
        get(),
        &target,
        impl_::is_linking_libpython_for_target(&target),
        std::io::stdout(),
    )
}

fn _add_python_framework_link_args(
    interpreter_config: &InterpreterConfig,
    triple: &Triple,
    link_libpython: bool,
    mut writer: impl std::io::Write,
) {
    if matches!(triple.operating_system, OperatingSystem::Darwin(_)) && link_libpython {
        if let Some(framework_prefix) = interpreter_config.python_framework_prefix.as_ref() {
            writeln!(writer, "cargo:rustc-link-arg=-Wl,-rpath,{framework_prefix}").unwrap();
        }
    }
}

/// Loads the configuration determined from the build environment.
///
/// Because this will never change in a given compilation run, this is cached in a `OnceLock`.
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: OnceLock<InterpreterConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let Some(interpreter_config) = InterpreterConfig::from_cargo_dep_env() else {
            panic!(
                "`pyo3_build_config::get()` requires a direct dependency on `pyo3` or `pyo3-ffi`"
            )
        };
        interpreter_config.expect("failed to parse PyO3 config")
    })
}

/// Helper to print a feature cfg with a minimum rust version required.
fn print_feature_cfg(minor_version_required: u32, cfg: &str) {
    let minor_version = rustc_minor_version().unwrap_or(0);

    if minor_version >= minor_version_required {
        println!("cargo:rustc-cfg={cfg}");
    }

    // rustc 1.80.0 stabilized `rustc-check-cfg` feature, don't emit before
    if minor_version >= 80 {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
}

/// Use certain features if we detect the compiler being used supports them.
///
/// Features may be removed or added as MSRV gets bumped or new features become available,
/// so this function is unstable.
#[doc(hidden)]
pub fn print_feature_cfgs() {
    print_feature_cfg(85, "fn_ptr_eq");
    print_feature_cfg(86, "from_bytes_with_nul_error");
}

/// Registers `pyo3`s config names as reachable cfg expressions
///
/// - <https://github.com/rust-lang/cargo/pull/13571>
/// - <https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html#rustc-check-cfg>
#[doc(hidden)]
pub fn print_expected_cfgs() {
    if rustc_minor_version().is_some_and(|version| version < 80) {
        // rustc 1.80.0 stabilized `rustc-check-cfg` feature, don't emit before
        return;
    }

    println!("cargo:rustc-check-cfg=cfg(Py_LIMITED_API)");
    println!("cargo:rustc-check-cfg=cfg(Py_GIL_DISABLED)");
    println!("cargo:rustc-check-cfg=cfg(PyPy)");
    println!("cargo:rustc-check-cfg=cfg(GraalPy)");
    println!("cargo:rustc-check-cfg=cfg(py_sys_config, values(\"Py_DEBUG\", \"Py_REF_DEBUG\", \"Py_TRACE_REFS\", \"COUNT_ALLOCS\"))");
    println!("cargo:rustc-check-cfg=cfg(pyo3_disable_reference_pool)");
    println!("cargo:rustc-check-cfg=cfg(pyo3_leak_on_drop_without_reference_pool)");

    // allow `Py_3_*` cfgs from the minimum supported version up to the
    // maximum minor version (+1 for development for the next)
    for i in impl_::MINIMUM_SUPPORTED_VERSION.minor..=impl_::ABI3_MAX_MINOR + 1 {
        println!("cargo:rustc-check-cfg=cfg(Py_3_{i})");
    }
}

/// Private exports used in PyO3's build.rs
///
/// Please don't use these - they could change at any time.
#[doc(hidden)]
pub mod pyo3_build_script_impl {
    use crate::{errors::Result, impl_::make_interpreter_config};

    use super::*;

    pub mod errors {
        pub use crate::errors::*;
    }
    pub use crate::impl_::{
        cargo_env_var, env_var, is_linking_libpython_for_target, make_cross_compile_config,
        target_triple_from_env, InterpreterConfig, PythonVersion,
    };
    pub enum BuildConfigSource {
        /// Config was provided by `PYO3_CONFIG_FILE`.
        ConfigFile,
        /// Config was found by an interpreter on the host system.
        Host,
        /// Config was configured by cross-compilation settings.
        CrossCompile,
    }

    pub struct BuildConfig {
        pub interpreter_config: InterpreterConfig,
        pub source: BuildConfigSource,
    }

    /// Gets the configuration for use from `pyo3-ffi`'s build script.
    pub fn resolve_build_config(target: &Triple) -> Result<BuildConfig> {
        #[allow(
            clippy::const_is_empty,
            reason = "CONFIG_FILE is generated in build.rs, content can vary"
        )]
        if let Some(mut interpreter_config) =
            InterpreterConfig::from_pyo3_config_file_env().transpose()?
        {
            interpreter_config.apply_default_lib_name_to_config_file(target);
            interpreter_config.generate_import_libs()?;
            Ok(BuildConfig {
                interpreter_config,
                source: BuildConfigSource::ConfigFile,
            })
        } else if let Some(interpreter_config) = make_cross_compile_config()? {
            Ok(BuildConfig {
                interpreter_config,
                source: BuildConfigSource::CrossCompile,
            })
        } else {
            // No config file, and no cross compile config, so fall back to trying to find an interpreter on the host system.
            let host_config = make_interpreter_config();
            Ok(BuildConfig {
                interpreter_config: host_config?,
                source: BuildConfigSource::Host,
            })
        }
    }

    /// Helper to generate an error message when the configured Python version is newer
    /// than PyO3's current supported version.
    pub struct MaximumVersionExceeded {
        message: String,
    }

    impl MaximumVersionExceeded {
        pub fn new(
            interpreter_config: &InterpreterConfig,
            supported_version: PythonVersion,
        ) -> Self {
            let implementation = match interpreter_config.implementation {
                PythonImplementation::CPython => "Python",
                PythonImplementation::PyPy => "PyPy",
                PythonImplementation::GraalPy => "GraalPy",
            };
            let version = &interpreter_config.version;
            let message = format!(
                "the configured {implementation} version ({version}) is newer than PyO3's maximum supported version ({supported_version})\n\
                = help: this package is being built with PyO3 version {current_version}\n\
                = help: check https://crates.io/crates/pyo3 for the latest PyO3 version available\n\
                = help: updating this package to the latest version of PyO3 may provide compatibility with this {implementation} version",
                current_version = env!("CARGO_PKG_VERSION")
            );
            Self { message }
        }

        pub fn add_help(&mut self, help: &str) {
            self.message.push_str("\n= help: ");
            self.message.push_str(help);
        }

        pub fn finish(self) -> String {
            self.message
        }
    }
}

fn rustc_minor_version() -> Option<u32> {
    static RUSTC_MINOR_VERSION: OnceLock<Option<u32>> = OnceLock::new();
    *RUSTC_MINOR_VERSION.get_or_init(|| {
        let rustc = env::var_os("RUSTC")?;
        let output = Command::new(rustc).arg("--version").output().ok()?;
        let version = core::str::from_utf8(&output.stdout).ok()?;
        let mut pieces = version.split('.');
        if pieces.next() != Some("rustc 1") {
            return None;
        }
        pieces.next()?.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_module_link_args() {
        let mut buf = Vec::new();

        // Does nothing on non-mac
        _add_extension_module_link_args(
            &Triple::from_str("x86_64-pc-windows-msvc").unwrap(),
            &mut buf,
        );
        assert_eq!(buf, Vec::new());

        _add_extension_module_link_args(
            &Triple::from_str("x86_64-apple-darwin").unwrap(),
            &mut buf,
        );
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "cargo:rustc-cdylib-link-arg=-undefined\n\
             cargo:rustc-cdylib-link-arg=dynamic_lookup\n"
        );

        buf.clear();
        _add_extension_module_link_args(
            &Triple::from_str("wasm32-unknown-emscripten").unwrap(),
            &mut buf,
        );
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "cargo:rustc-cdylib-link-arg=-sSIDE_MODULE=2\n\
             cargo:rustc-cdylib-link-arg=-sWASM_BIGINT\n"
        );
    }

    #[test]
    fn python_framework_link_args() {
        let mut buf = Vec::new();

        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion {
                major: 3,
                minor: 13,
            },
            shared: true,
            abi3: false,
            lib_name: None,
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags: BuildFlags::default(),
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: Some(
                "/Applications/Xcode.app/Contents/Developer/Library/Frameworks".to_string(),
            ),
        };
        // Does nothing on non-mac
        _add_python_framework_link_args(
            &interpreter_config,
            &Triple::from_str("x86_64-pc-windows-msvc").unwrap(),
            true,
            &mut buf,
        );
        assert_eq!(buf, Vec::new());

        _add_python_framework_link_args(
            &interpreter_config,
            &Triple::from_str("x86_64-apple-darwin").unwrap(),
            true,
            &mut buf,
        );
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "cargo:rustc-link-arg=-Wl,-rpath,/Applications/Xcode.app/Contents/Developer/Library/Frameworks\n"
        );
    }

    #[test]
    fn test_maximum_version_exceeded_formatting() {
        let interpreter_config = InterpreterConfig {
            implementation: PythonImplementation::CPython,
            version: PythonVersion {
                major: 3,
                minor: 13,
            },
            shared: true,
            abi3: false,
            lib_name: None,
            lib_dir: None,
            executable: None,
            pointer_width: None,
            build_flags: BuildFlags::default(),
            suppress_build_script_link_lines: false,
            extra_build_script_lines: vec![],
            python_framework_prefix: None,
        };
        let mut error = pyo3_build_script_impl::MaximumVersionExceeded::new(
            &interpreter_config,
            PythonVersion {
                major: 3,
                minor: 12,
            },
        );
        error.add_help("this is a help message");
        let error = error.finish();
        let expected = concat!("\
            the configured Python version (3.13) is newer than PyO3's maximum supported version (3.12)\n\
            = help: this package is being built with PyO3 version ", env!("CARGO_PKG_VERSION"), "\n\
            = help: check https://crates.io/crates/pyo3 for the latest PyO3 version available\n\
            = help: updating this package to the latest version of PyO3 may provide compatibility with this Python version\n\
            = help: this is a help message"
        );
        assert_eq!(error, expected);
    }
}
