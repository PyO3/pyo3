//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(since = "0.15.0", note = "use `fn __call__` instead of `#[call]`")]
pub const CALL_ATTRIBUTE: () = ();
