//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(
    since = "0.16.0",
    note = "implement a `__traverse__` `#[pymethod]` instead of using `gc` option"
)]
pub const PYCLASS_GC_OPTION: () = ();
