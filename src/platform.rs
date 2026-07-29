//! This module is to support platform compatibility with `no_std` environments.
#![allow(unused_imports)]

/// This prelude is intended to be used instead of the prelude from `std`.
pub(crate) mod prelude {
    pub use alloc::{
        borrow::ToOwned,
        boxed::Box,
        string::{String, ToString},
        vec::Vec,
    };
}

#[cfg(feature = "hashbrown")]
pub use hashbrown::{HashMap, HashSet};

// TODO conditionally import these based on "std" feature
#[cfg(not(feature = "hashbrown"))]
pub use std::collections::{HashMap, HashSet};
