//! This crate contains the implementation of the proc macro attributes

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![recursion_limit = "1024"]

// Listed first so that macros in this module are available in the rest of the crate.
#[macro_use]
mod utils;

mod attributes;
mod combine_errors;
mod derive_attributes;
mod frompyobject;
mod intopyobject;
#[cfg(feature = "experimental-inspect")]
mod introspection;
mod konst;
mod method;
mod module;
mod params;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
mod pyversions;
mod quotes;

pub use frompyobject::build_derive_from_pyobject;
pub use intopyobject::build_derive_into_pyobject;
pub use module::{pymodule_function_impl, pymodule_module_impl, PyModuleOptions};
pub use pyclass::{build_py_class, build_py_enum, PyClassArgs};
pub use pyfunction::{build_py_function, PyFunctionOptions};
pub use pyimpl::{build_py_methods, PyClassMethodsType};
pub use utils::get_doc;
