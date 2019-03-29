// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate contains the implementation of the proc macro attributes

#![recursion_limit = "1024"]

mod defs;
mod func;
mod method;
mod module;
mod pyclass;
mod pyfunction;
mod pyimpl;
mod pymethod;
mod pyproto;
mod utils;

pub use module::{add_fn_to_module, process_functions_in_module, py_init};
pub use pyclass::{build_py_class, PyClassArgs};
pub use pyfunction::PyFunctionAttr;
pub use pyimpl::{build_py_methods, impl_methods};
pub use pyproto::build_py_proto;
pub use utils::get_doc;
