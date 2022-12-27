//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(
    since = "0.18.0",
    note = "passing arbitrary arguments to `#[pyfunction()]` to specify the signature is being replaced by `#[pyo3(signature)]`"
)]
pub const PYFUNCTION_ARGUMENTS: () = ();

#[deprecated(
    since = "0.18.0",
    note = "the `#[args]` attribute for `#[methods]` is being replaced by `#[pyo3(signature)]`"
)]
pub const PYMETHODS_ARGS_ATTRIBUTE: () = ();
