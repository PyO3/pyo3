//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! This crate exposes functionality to be called from build scripts to simplify building crates
//! which depend on PyO3.
//!
//! It used internally by the PyO3 crate's build script to apply the same configuration.

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]

mod errors;
mod impl_;

use std::{env, process::Command, str::FromStr, sync::LazyLock};

pub use impl_::{
    cross_compiling_from_to, find_all_sysconfigdata, parse_sysconfigdata, BuildFlag, BuildFlags,
    CrossCompileConfig, InterpreterConfig, InterpreterConfigBuilder, PythonImplementation,
    PythonVersion, Triple,
};

#[cfg(feature = "resolve-config")]
use target_lexicon::Architecture;
use target_lexicon::{Architecture, OperatingSystem};

/// Adds all the [`#[cfg]` flags](index.html) to the current compilation.
///
/// This should be called from a build script.
///
/// The full list of attributes added are the following:
///
/// | Flag | Description |
/// | ---- | ----------- |
/// | `#[cfg(Py_3_8)]`, `#[cfg(Py_3_9)]`, `#[cfg(Py_3_10)]`, `#[cfg(Py_3_11)]`, ... | These attributes mark code only for a given Python version and up. For example, `#[cfg(Py_3_8)]` marks code which can run on Python 3.8 **and newer**. There is one attribute for each Python version currently supported by PyO3. |
/// | `#[cfg(Py_LIMITED_API)]` | This marks code which is run when compiling with PyO3's `abi3` feature enabled. |
/// | `#[cfg(Py_GIL_DISABLED)]` | This marks code which is run on the free-threaded interpreter. |
/// | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
/// | `#[cfg(GraalPy)]` | This marks code which is run when compiling for GraalPy. |
///
/// For examples of how to use these attributes,
#[doc = concat!("[see PyO3's guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/building-and-distribution/multiple-python-versions.html)")]
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
/// - wasm32-unknown-emscripten: for Rust <= 1.95, `-sSIDE_MODULE=2 -sWASM_BIGINT`
///
/// All other platforms currently are no-ops, however this may change as necessary
/// in future.
pub fn add_extension_module_link_args() {
    _add_extension_module_link_args(
        &impl_::target_triple_from_env(),
        std::io::stdout(),
        rustc_minor_version(),
    )
}

fn _add_extension_module_link_args(
    triple: &Triple,
    mut writer: impl std::io::Write,
    rustc_minor_version: Option<u32>,
) {
    if matches!(triple.operating_system, OperatingSystem::Darwin(_)) {
        writeln!(writer, "cargo:rustc-cdylib-link-arg=-undefined").unwrap();
        writeln!(writer, "cargo:rustc-cdylib-link-arg=dynamic_lookup").unwrap();
    } else if triple == &Triple::from_str("wasm32-unknown-emscripten").unwrap()
        && rustc_minor_version.is_some_and(|version| version < 95)
    {
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
    pyo3_build_script_impl::print_libpython_rpath_link_args(&target, get());
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
        if let Some(framework_prefix) = interpreter_config.python_framework_prefix() {
            writeln!(writer, "cargo:rustc-link-arg=-Wl,-rpath,{framework_prefix}").unwrap();
        }
    }
}

/// Loads the configuration determined from the build environment.
///
/// This function must be called from a build script, and requires a direct dependency on at
/// least one of `pyo3` or `pyo3-ffi`.
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: LazyLock<InterpreterConfig> = LazyLock::new(get_inner);
    &CONFIG
}

#[track_caller]
fn get_inner() -> InterpreterConfig {
    let Some(interpreter_config) = InterpreterConfig::from_cargo_dep_env() else {
        panic!("`pyo3_build_config::get()` requires a direct dependency on `pyo3` or `pyo3-ffi`")
    };
    interpreter_config.expect("failed to parse PyO3 config")
}

/// Registers `pyo3`s config names as reachable cfg expressions
///
/// - <https://github.com/rust-lang/cargo/pull/13571>
/// - <https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html#rustc-check-cfg>
#[doc(hidden)]
pub fn print_expected_cfgs() {
    println!("cargo:rustc-check-cfg=cfg(Py_LIMITED_API)");
    println!("cargo:rustc-check-cfg=cfg(Py_GIL_DISABLED)");
    println!("cargo:rustc-check-cfg=cfg(PyPy)");
    println!("cargo:rustc-check-cfg=cfg(GraalPy)");
    println!("cargo:rustc-check-cfg=cfg(RustPython)");
    println!("cargo:rustc-check-cfg=cfg(py_sys_config, values(\"Py_DEBUG\", \"Py_REF_DEBUG\", \"Py_TRACE_REFS\", \"COUNT_ALLOCS\"))");

    // allow `Py_3_*` cfgs from the minimum supported version up to the
    // maximum minor version (+1 for development for the next)
    for i in impl_::MINIMUM_SUPPORTED_VERSION.minor..=impl_::ABI3_MAX_MINOR + 1 {
        println!("cargo:rustc-check-cfg=cfg(Py_3_{i})");
    }

    // pyo3_dll cfg for raw-dylib linking on Windows
    let mut dll_names = vec!["python3".to_string(), "python3_d".to_string()];
    for i in impl_::MINIMUM_SUPPORTED_VERSION.minor..=impl_::ABI3_MAX_MINOR + 1 {
        dll_names.push(format!("python3{i}"));
        dll_names.push(format!("python3{i}_d"));
        if i >= 13 {
            dll_names.push(format!("python3{i}t"));
            dll_names.push(format!("python3{i}t_d"));
        }
    }
    // PyPy DLL names (libpypy3.X-c.dll)
    for i in
        impl_::MINIMUM_SUPPORTED_VERSION_PYPY.minor..=impl_::MAXIMUM_SUPPORTED_VERSION_PYPY.minor
    {
        dll_names.push(format!("libpypy3.{i}-c"));
    }
    let values = dll_names
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ");
    println!("cargo:rustc-check-cfg=cfg(pyo3_dll, values({values}))");
}

/// Private exports used in PyO3's build.rs
///
/// Please don't use these - they could change at any time.
#[doc(hidden)]
pub mod pyo3_build_script_impl {
    use crate::{
        errors::Result,
        impl_::{make_cross_compile_config, make_interpreter_config},
    };

    use super::*;

    pub mod errors {
        pub use crate::errors::*;
    }
    pub use crate::impl_::{
        cargo_env_var, env_var, is_linking_libpython_for_target, target_triple_from_env,
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
        if let Some(interpreter_config) =
            InterpreterConfig::from_pyo3_config_file_env(target).transpose()?
        {
            Ok(BuildConfig {
                interpreter_config,
                source: BuildConfigSource::ConfigFile,
            })
        } else if let Some(interpreter_config) = make_cross_compile_config(target)? {
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
            let implementation = match interpreter_config.implementation() {
                PythonImplementation::CPython => "Python",
                PythonImplementation::PyPy => "PyPy",
                PythonImplementation::GraalPy => "GraalPy",
                PythonImplementation::RustPython => "RustPython",
            };
            let version = interpreter_config.version();
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

    /// Detects features which `pyo3` and `pyo3-ffi` depend upon internally, and prints the appropriate
    /// `cargo:rustc-cfg` and `cargo:rustc-check-cfg` directives to enable them.
    pub fn print_feature_cfgs() {
        print_feature_cfg(84, "const_is_null");
        print_feature_cfg(85, "fn_ptr_eq");
        print_feature_cfg(86, "from_bytes_with_nul_error");
        print_feature_cfg(95, "cfg_select");
    }

    /// Helper to print a feature cfg with a minimum rust version required.
    fn print_feature_cfg(minor_version_required: u32, cfg: &str) {
        println!("cargo:rustc-check-cfg=cfg({cfg})");

        let minor_version = rustc_minor_version().unwrap_or(0);
        if minor_version >= minor_version_required {
            println!("cargo:rustc-cfg={cfg}");
        }
    }

    /// Emit libpython rpath link args if appropriate for the target and interpreter config.
    ///
    /// This form exists for pyo3-ffi where `get()` cannot be called.
    pub fn print_libpython_rpath_link_args(
        target: &Triple,
        interpreter_config: &InterpreterConfig,
    ) {
        let is_linking_libpython = is_linking_libpython_for_target(target);
        let is_wasm = matches!(
            target.architecture,
            Architecture::Wasm32 | Architecture::Wasm64
        );
        let is_emscripten = target.operating_system == target_lexicon::OperatingSystem::Emscripten;
        // webassembly targets generally don't support rpath, emscripten is the only exception currently aware of:
        // https://github.com/emscripten-core/emscripten/issues/22126
        if is_linking_libpython && (!is_wasm || is_emscripten) {
            if let Some(lib_dir) = interpreter_config.lib_dir() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
            }
        }
    }
}

fn rustc_minor_version() -> Option<u32> {
    static RUSTC_MINOR_VERSION: LazyLock<Option<u32>> = LazyLock::new(|| {
        let rustc = env::var_os("RUSTC")?;
        let output = Command::new(rustc).arg("--version").output().ok()?;
        let version = core::str::from_utf8(&output.stdout).ok()?;
        let mut pieces = version.split('.');
        if pieces.next() != Some("rustc 1") {
            return None;
        }
        pieces.next()?.parse().ok()
    });
    *RUSTC_MINOR_VERSION
}

#[cfg(test)]
#[expect(deprecated, reason = "accessing config directly")]
mod tests {
    use crate::impl_::escape;

    use super::*;

    #[test]
    fn extension_module_link_args() {
        let mut buf = Vec::new();

        // Does nothing on non-mac
        _add_extension_module_link_args(
            &Triple::from_str("x86_64-pc-windows-msvc").unwrap(),
            &mut buf,
            None,
        );
        assert_eq!(buf, Vec::new());

        _add_extension_module_link_args(
            &Triple::from_str("x86_64-apple-darwin").unwrap(),
            &mut buf,
            None,
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
            Some(94),
        );
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "cargo:rustc-cdylib-link-arg=-sSIDE_MODULE=2\n\
             cargo:rustc-cdylib-link-arg=-sWASM_BIGINT\n"
        );
        buf.clear();
        _add_extension_module_link_args(
            &Triple::from_str("wasm32-unknown-emscripten").unwrap(),
            &mut buf,
            Some(95),
        );
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "");
    }

    #[test]
    fn python_framework_link_args() {
        let mut buf = Vec::new();
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY313;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .python_framework_prefix(
                "/Applications/Xcode.app/Contents/Developer/Library/Frameworks".to_string(),
            )
            .finalize()
            .unwrap();

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
        let implementation = PythonImplementation::CPython;
        let version = PythonVersion::PY313;
        let interpreter_config = InterpreterConfigBuilder::new(implementation, version)
            .finalize()
            .unwrap();
        let mut error = pyo3_build_script_impl::MaximumVersionExceeded::new(
            &interpreter_config,
            PythonVersion::PY312,
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

    #[test]
    fn test_interpreter_config_from_cargo_env() {
        // There should be no other tests or config in the environment
        assert!(InterpreterConfig::from_cargo_dep_env().is_none());

        let interpreter_config =
            InterpreterConfigBuilder::new(PythonImplementation::CPython, PythonVersion::PY313)
                .finalize()
                .unwrap();
        let mut buf = Vec::new();
        interpreter_config.to_writer(&mut buf).unwrap();
        let config_string = escape(&buf);
        // SAFETY: no other tests use `crate::get()`
        unsafe { std::env::set_var(InterpreterConfig::PYO3_FFI_CONFIG_ENV_VAR, &config_string) };

        assert_eq!(get_inner(), interpreter_config);

        // Repeat with PyO3 env var
        // SAFETY: no other tests use `crate::get()`
        unsafe {
            std::env::remove_var(InterpreterConfig::PYO3_FFI_CONFIG_ENV_VAR);
            std::env::set_var(InterpreterConfig::PYO3_CONFIG_ENV_VAR, &config_string)
        }

        assert_eq!(get_inner(), interpreter_config);
    }
}
