//! This module is to support platform compatiblity with `no_std` environments.

/// This prelude is intended to be used instead of the prelude from `std`.
#[allow(unused_imports)]
pub(crate) mod prelude {
    pub use alloc::{
        borrow::ToOwned,
        boxed::Box,
        string::{String, ToString},
        vec::Vec,
    };

    // TODO find a `no_std` replacement for eprintln
    pub use std::eprintln;
}
