//! This module contains conversions between various Rust object and their representation in Python.

mod array;
#[cfg(feature = "indexmap")]
#[cfg_attr(docsrs, doc(cfg(feature = "indexmap")))]
pub mod indexmap;
mod osstr;
mod path;
