//! Runtime inspection of Python data structures.
//!
//! This module provides APIs to access information on Python data structures (classes, builtins) at runtime from Rust.
//! These APIs can be used to generate documentation, interface files (.pyi), etc.

pub mod types;
pub mod fields;
pub mod classes;
pub mod modules;
pub mod interface;
