//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! This crate exposes functionality to be called from build scripts to simplify building crates
//! which depend on PyO3.
//!
//! It used internally by the PyO3 crate's build script to apply the same configuration.

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]

mod errors;
mod impl_;

#[cfg(feature = "resolve-config")]
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

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
/// | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
/// | `#[cfg(GraalPy)]` | This marks code which is run when compiling for GraalPy. |
///
/// For examples of how to use these attributes,
#[doc = concat!("[see PyO3's guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/building-and-distribution/multiple_python_versions.html)")]
/// .
#[cfg(feature = "resolve-config")]
pub fn use_pyo3_cfgs() {
    print_expected_cfgs();
    for cargo_command in get().build_script_outputs() {
        println!("{cargo_command}")
    }
}

/// Adds linker arguments suitable for PyO3's `extension-module` feature.
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

/// Adds linker arguments suitable for linking against the Python framework on macOS.
///
/// This should be called from a build script.
///
/// The following link flags are added:
/// - macOS: `-Wl,-rpath,<framework_prefix>`
///
/// All other platforms currently are no-ops.
#[cfg(feature = "resolve-config")]
pub fn add_python_framework_link_args() {
    let interpreter_config = pyo3_build_script_impl::resolve_interpreter_config().unwrap();
    _add_python_framework_link_args(
        &interpreter_config,
        &impl_::target_triple_from_env(),
        impl_::is_linking_libpython(),
        std::io::stdout(),
    )
}

#[cfg(feature = "resolve-config")]
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
#[cfg(feature = "resolve-config")]
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: OnceLock<InterpreterConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        // Check if we are in a build script and cross compiling to a different target.
        let cross_compile_config_path = resolve_cross_compile_config_path();
        let cross_compiling = cross_compile_config_path
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false);

        // CONFIG_FILE is generated in build.rs, so it's content can vary
        #[allow(unknown_lints, clippy::const_is_empty)]
        if let Some(interpreter_config) = InterpreterConfig::from_cargo_dep_env() {
            interpreter_config
        } else if !CONFIG_FILE.is_empty() {
            InterpreterConfig::from_reader(Cursor::new(CONFIG_FILE))
        } else if cross_compiling {
            InterpreterConfig::from_path(cross_compile_config_path.as_ref().unwrap())
        } else {
            InterpreterConfig::from_reader(Cursor::new(HOST_CONFIG))
        }
        .expect("failed to parse PyO3 config")
    })
}

/// Build configuration provided by `PYO3_CONFIG_FILE`. May be empty if env var not set.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const CONFIG_FILE: &str = include_str!(concat!(env!("OUT_DIR"), "/pyo3-build-config-file.txt"));

/// Build configuration discovered by `pyo3-build-config` build script. Not aware of
/// cross-compilation settings.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const HOST_CONFIG: &str = include_str!(concat!(env!("OUT_DIR"), "/pyo3-build-config.txt"));

/// Returns the path where PyO3's build.rs writes its cross compile configuration.
///
/// The config file will be named `$OUT_DIR/<triple>/pyo3-build-config.txt`.
///
/// Must be called from a build script, returns `None` if not.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
fn resolve_cross_compile_config_path() -> Option<PathBuf> {
    env::var_os("TARGET").map(|target| {
        let mut path = PathBuf::from(env!("OUT_DIR"));
        path.push(Path::new(&target));
        path.push("pyo3-build-config.txt");
        path
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
    print_feature_cfg(79, "c_str_lit");
    // Actually this is available on 1.78, but we should avoid
    // https://github.com/rust-lang/rust/issues/124651 just in case
    print_feature_cfg(79, "diagnostic_namespace");
    print_feature_cfg(83, "io_error_more");
    print_feature_cfg(83, "mut_ref_in_const_fn");
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
    #[cfg(feature = "resolve-config")]
    use crate::errors::{Context, Result};

    #[cfg(feature = "resolve-config")]
    use super::*;

    pub mod errors {
        pub use crate::errors::*;
    }
    pub use crate::impl_::{
        cargo_env_var, env_var, is_linking_libpython, make_cross_compile_config, InterpreterConfig,
        PythonVersion,
    };

    /// Gets the configuration for use from PyO3's build script.
    ///
    /// Differs from .get() above only in the cross-compile case, where PyO3's build script is
    /// required to generate a new config (as it's the first build script which has access to the
    /// correct value for CARGO_CFG_TARGET_OS).
    #[cfg(feature = "resolve-config")]
    pub fn resolve_interpreter_config() -> Result<InterpreterConfig> {
        // CONFIG_FILE is generated in build.rs, so it's content can vary
        #[allow(unknown_lints, clippy::const_is_empty)]
        if !CONFIG_FILE.is_empty() {
            let mut interperter_config = InterpreterConfig::from_reader(Cursor::new(CONFIG_FILE))?;
            interperter_config.generate_import_libs()?;
            Ok(interperter_config)
        } else if let Some(interpreter_config) = make_cross_compile_config()? {
            // This is a cross compile and need to write the config file.
            let path = resolve_cross_compile_config_path()
                .expect("resolve_interpreter_config() must be called from a build script");
            let parent_dir = path.parent().ok_or_else(|| {
                format!(
                    "failed to resolve parent directory of config file {}",
                    path.display()
                )
            })?;
            std::fs::create_dir_all(parent_dir).with_context(|| {
                format!(
                    "failed to create config file directory {}",
                    parent_dir.display()
                )
            })?;
            interpreter_config.to_writer(&mut std::fs::File::create(&path).with_context(
                || format!("failed to create config file at {}", path.display()),
            )?)?;
            Ok(interpreter_config)
        } else {
            InterpreterConfig::from_reader(Cursor::new(HOST_CONFIG))
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

    #[cfg(feature = "resolve-config")]
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
}
