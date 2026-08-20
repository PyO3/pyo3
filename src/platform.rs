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
pub use hashbrown::{HashMap, HashSet};

#[cfg(all(not(feature = "hashbrown"), wip_feature_std))]
pub use std::collections::{HashMap, HashSet};

#[cfg(feature = "hashbrown")]
#[derive(Default)]
pub struct DefaultHashBuilder(hashbrown::DefaultHashBuilder);

#[cfg(all(not(feature = "hashbrown"), wip_feature_std))]
#[derive(Default)]
pub struct DefaultHashBuilder(core::hash::BuildHasherDefault<std::hash::DefaultHasher>);

impl core::hash::BuildHasher for DefaultHashBuilder {
    #[cfg(feature = "hashbrown")]
    type Hasher = hashbrown::DefaultHasher;
    #[cfg(all(not(feature = "hashbrown"), wip_feature_std))]
    type Hasher = std::hash::DefaultHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        self.0.build_hasher()
    }
}

#[cfg(all(not(feature = "hashbrown"), not(wip_feature_std)))]
compile_error!("Please enable at least one of the following features: hashbrown, std");
