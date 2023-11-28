//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(since = "0.20.0", note = "use `#[new]` instead of `#[__new__]`")]
pub const PYMETHODS_NEW_DEPRECATED_FORM: () = ();
