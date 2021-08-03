//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! The public APIs exposed, [`use_pyo3_cfgs`] and [`add_extension_module_link_args`] are intended
//! to be called from build scripts to simplify building crates which depend on PyO3.

#[doc(hidden)]
pub mod errors;
mod impl_;

use std::{ffi::OsString, path::Path};

use once_cell::sync::OnceCell;

// Used in PyO3's build.rs
#[doc(hidden)]
pub use impl_::{
    cargo_env_var, env_var, find_interpreter, get_config_from_interpreter, make_interpreter_config,
    InterpreterConfig, PythonImplementation, PythonVersion,
};

/// Reads the configuration written by PyO3's build.rs
///
/// Because this will never change in a given compilation run, this is cached in a `once_cell`.
#[doc(hidden)]
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: OnceCell<InterpreterConfig> = OnceCell::new();
    CONFIG.get_or_init(|| {
        let config_path = std::env::var_os("PYO3_CONFIG_FILE")
            .unwrap_or_else(|| OsString::from(DEFAULT_CONFIG_PATH));
        let config_file = std::fs::File::open(DEFAULT_CONFIG_PATH).expect(&format!(
            "failed to open PyO3 config file at {}",
            Path::new(&config_path).display()
        ));
        let reader = std::io::BufReader::new(config_file);
        InterpreterConfig::from_reader(reader).expect("failed to parse config file")
    })
}

/// Path where PyO3's build.rs will write configuration by default.
#[doc(hidden)]
pub const DEFAULT_CONFIG_PATH: &str = concat!(env!("OUT_DIR"), "/pyo3-build-config.txt");

/// Adds all the [`#[cfg]` flags](index.html) to the current compilation.
///
/// This should be called from a build script.
///
/// The full list of attributes added are the following:
///
/// | Flag | Description |
/// | ---- | ----------- |
/// | `#[cfg(Py_3_6)]`, `#[cfg(Py_3_7)]`, `#[cfg(Py_3_8)]`, `#[cfg(Py_3_9)]`, `#[cfg(Py_3_10)]` | These attributes mark code only for a given Python version and up. For example, `#[cfg(Py_3_6)]` marks code which can run on Python 3.6 **and newer**. |
/// | `#[cfg(Py_LIMITED_API)]` | This marks code which is run when compiling with PyO3's `abi3` feature enabled. |
/// | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
///
/// For examples of how to use these attributes, [see PyO3's guide](https://pyo3.rs/latest/building_and_distribution/multiple_python_versions.html).
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
    if cargo_env_var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
