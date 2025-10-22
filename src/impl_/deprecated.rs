pub struct DeprecatedFromPyObjectBlanket<const IS_CLONE: bool> {}

impl DeprecatedFromPyObjectBlanket<true> {
    #[deprecated(
        since = "0.28.0",
        note = "Implicit by value extraction of pyclasses is deprecated. Use `from_py_object` to keep the current behaviour or `skip_from_py_object` to opt-out."
    )]
    pub const MSG: () = ();
}

impl DeprecatedFromPyObjectBlanket<false> {
    pub const MSG: () = ();
}
