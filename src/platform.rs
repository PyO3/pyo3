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

    // TODO find a `no_std` replacement for eprintln
    pub use std::eprintln;
}

#[cfg(feature = "hashbrown")]
pub use hashbrown::{DefaultHasher, HashMap, HashSet};

#[cfg(all(not(feature = "hashbrown"), wip_feature_std))]
pub use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};

#[cfg(all(not(feature = "hashbrown"), not(wip_feature_std)))]
compile_error!("Please enable at least one of the following features: hashbrown, std");
