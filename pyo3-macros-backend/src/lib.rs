//! This crate contains the implementation of the proc macro attributes

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![recursion_limit = "1024"]

// Listed first so that macros in this module are available in the rest of the crate.
#[macro_use]
mod utils;

mod attributes;
mod deprecations;
mod frompyobject;
mod konst;
mod method;
mod module;
mod params;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
mod intopydict;

pub use frompyobject::build_derive_from_pyobject;
pub use module::{process_functions_in_module, pymodule_impl, PyModuleOptions};
pub use pyclass::{build_py_class, build_py_enum, PyClassArgs};
pub use pyfunction::{build_py_function, PyFunctionOptions};
pub use pyimpl::{build_py_methods, PyClassMethodsType};
pub use utils::get_doc;
pub use intopydict::build_derive_into_pydict;
