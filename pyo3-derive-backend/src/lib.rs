// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate contains the implementation of the proc macro attributes

#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

pub mod py_class;
pub mod py_impl;
pub mod py_proto;
pub mod py_method;
pub mod args;
pub mod defs;
pub mod func;
pub mod method;
pub mod module;
pub mod utils;
