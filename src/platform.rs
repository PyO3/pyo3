//! This module is to support platform compatibility with `no_std` environments.
#![allow(unused_imports)]

#[cfg(feature = "hashbrown")]
pub use hashbrown::{HashMap, HashSet};

// TODO conditionally import these based on "std" feature
#[cfg(not(feature = "hashbrown"))]
pub use std::collections::{HashMap, HashSet};
