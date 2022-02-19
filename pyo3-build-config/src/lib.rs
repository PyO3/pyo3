//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! This crate exposes functionality to be called from build scripts to simplify building crates
//! which depend on PyO3.
//!
//! It used internally by the PyO3 crate's build script to apply the same configuration.

mod errors;
mod impl_;

#[cfg(feature = "resolve-config")]
use std::io::Cursor;
use std::{env, process::Command};

#[cfg(feature = "resolve-config")]
use once_cell::sync::OnceCell;

pub use impl_::{
    cross_compiling, find_all_sysconfigdata, parse_sysconfigdata, BuildFlag, BuildFlags,
    CrossCompileConfig, InterpreterConfig, PythonImplementation, PythonVersion,
};

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
///
/// For examples of how to use these attributes, [see PyO3's guide](https://pyo3.rs/latest/building_and_distribution/multiple_python_versions.html).
#[cfg(feature = "resolve-config")]
pub fn use_pyo3_cfgs() {
    get().emit_pyo3_cfgs();
}

/// Adds linker arguments (for macOS) suitable for PyO3's `extension-module` feature.
///
/// This should be called from a build script.
///
/// This is currently a no-op on non-macOS platforms, however may emit additional linker arguments
/// in future if deemed necessarys.
pub fn add_extension_module_link_args() {
    _add_extension_module_link_args(
        &impl_::cargo_env_var("CARGO_CFG_TARGET_OS").unwrap(),
        std::io::stdout(),
    )
}

fn _add_extension_module_link_args(target_os: &str, mut writer: impl std::io::Write) {
    if target_os == "macos" {
        writeln!(writer, "cargo:rustc-cdylib-link-arg=-undefined").unwrap();
        writeln!(writer, "cargo:rustc-cdylib-link-arg=dynamic_lookup").unwrap();
    }
}

/// Loads the configuration determined from the build environment.
///
/// Because this will never change in a given compilation run, this is cached in a `once_cell`.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: OnceCell<InterpreterConfig> = OnceCell::new();
    CONFIG.get_or_init(|| {
        if !CONFIG_FILE.is_empty() {
            InterpreterConfig::from_reader(Cursor::new(CONFIG_FILE))
        } else if !ABI3_CONFIG.is_empty() {
            Ok(abi3_config())
        } else if impl_::any_cross_compiling_env_vars_set() {
            InterpreterConfig::from_path(DEFAULT_CROSS_COMPILE_CONFIG_PATH)
        } else {
            InterpreterConfig::from_reader(Cursor::new(HOST_CONFIG))
        }
        .expect("failed to parse PyO3 config file")
    })
}

/// Path where PyO3's build.rs will write configuration by default.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const DEFAULT_CROSS_COMPILE_CONFIG_PATH: &str =
    concat!(env!("OUT_DIR"), "/pyo3-cross-compile-config.txt");

/// Build configuration provided by `PYO3_CONFIG_FILE`. May be empty if env var not set.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const CONFIG_FILE: &str = include_str!(concat!(env!("OUT_DIR"), "/pyo3-build-config-file.txt"));

/// Build configuration set if abi3 features enabled and `PYO3_NO_PYTHON` env var present. Empty if
/// not both present.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const ABI3_CONFIG: &str = include_str!(concat!(env!("OUT_DIR"), "/pyo3-build-config-abi3.txt"));

/// Build configuration discovered by `pyo3-build-config` build script. Not aware of
/// cross-compilation settings.
#[doc(hidden)]
#[cfg(feature = "resolve-config")]
const HOST_CONFIG: &str = include_str!(concat!(env!("OUT_DIR"), "/pyo3-build-config.txt"));

#[cfg(feature = "resolve-config")]
fn abi3_config() -> InterpreterConfig {
    let mut interpreter_config = InterpreterConfig::from_reader(Cursor::new(ABI3_CONFIG))
        .expect("failed to parse hardcoded PyO3 abi3 config");
    // If running from a build script on Windows, tweak the hardcoded abi3 config to contain
    // the standard lib name (this is necessary so that abi3 extension modules using
    // PYO3_NO_PYTHON on Windows can link)
    if std::env::var("CARGO_CFG_TARGET_OS").map_or(false, |target_os| target_os == "windows") {
        assert_eq!(interpreter_config.lib_name, None);
        interpreter_config.lib_name = Some("python3".to_owned())
    }
    interpreter_config
}

/// Use certain features if we detect the compiler being used supports them.
///
/// Features may be removed or added as MSRV gets bumped or new features become available,
/// so this function is unstable.
#[doc(hidden)]
pub fn print_feature_cfgs() {
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

    let rustc_minor_version = rustc_minor_version().unwrap_or(0);

    // Enable use of const generics on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=min_const_generics");
    }

    // Enable use of std::ptr::addr_of! on Rust 1.51 and greater
    if rustc_minor_version >= 51 {
        println!("cargo:rustc-cfg=addr_of");
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
    use std::path::Path;

    #[cfg(feature = "resolve-config")]
    use super::*;

    pub mod errors {
        pub use crate::errors::*;
    }
    pub use crate::impl_::{
        cargo_env_var, env_var, make_cross_compile_config, InterpreterConfig, PythonVersion,
    };

    /// Gets the configuration for use from PyO3's build script.
    ///
    /// Differs from .get() above only in the cross-compile case, where PyO3's build script is
    /// required to generate a new config (as it's the first build script which has access to the
    /// correct value for CARGO_CFG_TARGET_OS).
    #[cfg(feature = "resolve-config")]
    pub fn resolve_interpreter_config() -> Result<InterpreterConfig> {
        if !CONFIG_FILE.is_empty() {
            InterpreterConfig::from_reader(Cursor::new(CONFIG_FILE))
        } else if !ABI3_CONFIG.is_empty() {
            Ok(abi3_config())
        } else if let Some(interpreter_config) = impl_::make_cross_compile_config()? {
            // This is a cross compile and need to write the config file.
            let path = Path::new(DEFAULT_CROSS_COMPILE_CONFIG_PATH);
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
            interpreter_config.to_writer(&mut std::fs::File::create(&path).with_context(
                || format!("failed to create config file at {}", path.display()),
            )?)?;
            Ok(interpreter_config)
        } else {
            InterpreterConfig::from_reader(Cursor::new(HOST_CONFIG))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_module_link_args() {
        let mut buf = Vec::new();

        // Does nothing on non-mac
        _add_extension_module_link_args("windows", &mut buf);
        assert_eq!(buf, Vec::new());

        _add_extension_module_link_args("macos", &mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "cargo:rustc-cdylib-link-arg=-undefined\n\
             cargo:rustc-cdylib-link-arg=dynamic_lookup\n"
        );
    }
}
