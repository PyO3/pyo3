//! This module is to support platform compatibility with `no_std` environments.
#![allow(unused_imports)]
#![doc(hidden)]

/// This prelude is intended to be used instead of the prelude from `std`.
pub(crate) mod prelude {
    pub use alloc::{
        borrow::ToOwned,
        boxed::Box,
        string::{String, ToString},
        vec::Vec,
    };
}

pub mod sync;

#[cfg(wip_feature_std)]
#[derive(Default)]
pub struct DefaultHashBuilder(core::hash::BuildHasherDefault<std::hash::DefaultHasher>);

#[cfg(all(not(wip_feature_std), feature = "hashbrown"))]
pub struct DefaultHashBuilder(hashbrown::DefaultHashBuilder);

#[cfg(all(not(wip_feature_std), feature = "hashbrown"))]
impl Default for DefaultHashBuilder {
    fn default() -> Self {
        use once_cell::race::OnceBox;
        static CACHED: OnceBox<hashbrown::DefaultHashBuilder> = OnceBox::new();
        Self(
            CACHED
                .get_or_init(|| alloc::boxed::Box::new(hashbrown::DefaultHashBuilder::default()))
                .clone(),
        )
    }
}

#[cfg(any(wip_feature_std, feature = "hashbrown"))]
impl core::hash::BuildHasher for DefaultHashBuilder {
    #[cfg(wip_feature_std)]
    type Hasher = std::hash::DefaultHasher;

    #[cfg(all(not(wip_feature_std), feature = "hashbrown"))]
    type Hasher = <hashbrown::DefaultHashBuilder as core::hash::BuildHasher>::Hasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        self.0.build_hasher()
    }
}

pub mod collections {
    #[cfg(feature = "hashbrown")]
    pub use hashbrown::{HashMap, HashSet};

    #[cfg(all(not(feature = "hashbrown"), wip_feature_std))]
    pub use std::collections::{HashMap, HashSet};

    #[cfg(all(not(feature = "hashbrown"), not(wip_feature_std)))]
    compile_error!("Please enable at least one of the following features: hashbrown, std");
}

pub mod thread;
