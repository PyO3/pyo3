//! Holding place for code which is not intended to be reachable from outside of PyO3.

#[macro_use]
pub(crate) mod macros;

pub(crate) mod get_slot;
pub(crate) mod pyclass_init;
pub(crate) mod state;
