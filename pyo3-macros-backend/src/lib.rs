// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate contains the implementation of the proc macro attributes

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "1024"]
// Listed first so that macros in this module are available in the rest of the crate.
#[macro_use]
mod utils;

mod attributes;
mod defs;
mod deprecations;
mod from_pyobject;
mod konst;
mod method;
mod module;
mod params;
mod proto_method;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
mod pyproto;

pub use from_pyobject::build_derive_from_pyobject;
pub use module::{process_functions_in_module, py_init, PyModuleOptions};
pub use pyclass::{build_py_class, PyClassArgs};
pub use pyfunction::{build_py_function, PyFunctionOptions};
pub use pyimpl::{build_py_methods, PyClassMethodsType};
pub use pyproto::build_py_proto;
pub use utils::get_doc;
