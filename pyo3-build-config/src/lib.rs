//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! The only public API currently exposed is [`use_pyo3_cfgs`], which is intended to be used in
//! build scripts to add a standard set of `#[cfg]` attributes for handling multiple Python
//! versions.
//!
//! The full list of attributes added are the following:
//!
//! | Flag | Description |
//! | ---- | ----------- |
//! | `#[cfg(Py_3_6)]`, `#[cfg(Py_3_7)]`, `#[cfg(Py_3_8)]`, `#[cfg(Py_3_9)]`, `#[cfg(Py_3_10)]` | These attributes mark code only for a given Python version and up. For example, `#[cfg(Py_3_6)]` marks code which can run on Python 3.6 **and newer**. |
//! | `#[cfg(Py_LIMITED_API)]` | This marks code which is run when compiling with PyO3's `abi3` feature enabled. |
//! | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
//!
//! For examples of how to use these attributes, [see PyO3's guide](https://pyo3.rs/latest/building_and_distribution/multiple_python_versions.html).

#[doc(hidden)]
pub mod errors;
mod impl_;

use once_cell::sync::OnceCell;

pub use impl_::{
    find_interpreter, get_config_from_interpreter, InterpreterConfig, PythonImplementation,
    PythonVersion,
};

// Used in PyO3's build.rs
#[doc(hidden)]
pub use impl_::make_interpreter_config;

/// Reads the configuration written by PyO3's build.rs
///
/// Because this will never change in a given compilation run, this is cached in a `once_cell`.
#[doc(hidden)]
pub fn get() -> &'static InterpreterConfig {
    static CONFIG: OnceCell<InterpreterConfig> = OnceCell::new();
    CONFIG.get_or_init(|| {
        let config_file = std::fs::File::open(PATH).expect("config file missing");
        let reader = std::io::BufReader::new(config_file);
        InterpreterConfig::from_reader(reader).expect("failed to parse config file")
    })
}

/// Path where PyO3's build.rs will write configuration.
#[doc(hidden)]
pub const PATH: &str = concat!(env!("OUT_DIR"), "/pyo3-build-config.txt");

pub fn use_pyo3_cfgs() {
    get().emit_pyo3_cfgs();
}
