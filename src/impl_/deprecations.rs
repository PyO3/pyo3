//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(
    since = "0.19.0",
    note = "put `text_signature` on `#[new]` instead of `#[pyclass]`"
)]
pub const PYCLASS_TEXT_SIGNATURE: () = ();

#[deprecated(since = "0.20.0", note = "use `#[new]` instead of `#[__new__]`")]
pub const PYMETHODS_NEW_DEPRECATED_FORM: () = ();
