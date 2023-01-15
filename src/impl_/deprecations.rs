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

#[deprecated(
    since = "0.18.0",
    note = "required arguments after an `Option<_>` argument are ambiguous and being phased out\n= help: add a `#[pyo3(signature)]` annotation on this function to unambiguously specify the default values for all optional parameters"
)]
pub const REQUIRED_ARGUMENT_AFTER_OPTION: () = ();
