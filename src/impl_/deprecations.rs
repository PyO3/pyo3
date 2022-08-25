//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(
    since = "0.16.0",
    note = "implement a `__traverse__` `#[pymethod]` instead of using `gc` option"
)]
pub const PYCLASS_GC_OPTION: () = ();

#[deprecated(
    since = "0.17.0",
    note = "required arguments after an `Option<_>` argument are ambiguous and being phased out\n= help: add a `#[pyo3(signature)]` annotation on this function to unambiguously specify the default values for all optional parameters"
)]
pub const REQUIRED_ARGUMENT_AFTER_OPTION: () = ();
