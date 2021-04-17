//! Internals of PyO3 which are accessed by code expanded from PyO3's procedural macros. Usage of
//! any of these APIs in downstream code is implicitly acknowledging that these APIs may change at
//! any time without documentation in the CHANGELOG and without breaking semver guarantees.

/// Symbols to represent deprecated uses of PyO3's macros.
pub mod deprecations {
    #[doc(hidden)]
    #[deprecated(
        since = "0.14.0",
        note = "use `#[pyo3(name = \"...\")]` instead of `#[name = \"...\"]`"
    )]
    pub const NAME_ATTRIBUTE: () = ();
}
