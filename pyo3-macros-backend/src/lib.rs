// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate contains the implementation of the proc macro attributes

#![warn(elided_lifetimes_in_paths, unused_lifetimes)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![recursion_limit = "1024"]

// Listed first so that macros in this module are available in the rest of the crate.
#[macro_use]
mod utils;

mod attributes;
#[cfg(feature = "pyproto")]
mod defs;
mod deprecations;
mod frompyobject;
mod konst;
mod method;
mod module;
mod params;
#[cfg(feature = "pyproto")]
mod proto_method;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
#[cfg(feature = "pyproto")]
mod pyproto;
mod wrap;
mod stubs;

pub use frompyobject::build_derive_from_pyobject;
pub use module::{process_functions_in_module, pymodule_impl, PyModuleOptions};
pub use pyclass::{build_py_class, build_py_enum, PyClassArgs};
pub use pyfunction::{build_py_function, PyFunctionOptions};
pub use pyimpl::{build_py_methods, PyClassMethodsType};
#[cfg(feature = "pyproto")]
pub use pyproto::build_py_proto;
pub use utils::get_doc;
pub use wrap::{wrap_pyfunction_impl, wrap_pymodule_impl, WrapPyFunctionArgs};
