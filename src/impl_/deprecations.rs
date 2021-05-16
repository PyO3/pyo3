//! Symbols used to denote deprecated usages of PyO3's proc macros.

#[deprecated(
    since = "0.14.0",
    note = "use `#[pyo3(name = \"...\")]` instead of `#[name = \"...\"]`"
)]
pub const NAME_ATTRIBUTE: () = ();

#[deprecated(
    since = "0.14.0",
    note = "use `#[pyfn(m)] #[pyo3(name = \"...\")]` instead of `#[pyfn(m, \"...\")]`"
)]
pub const PYFN_NAME_ARGUMENT: () = ();
