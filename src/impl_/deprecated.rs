pub struct HasAutomaticFromPyObject<const IS_CLONE: bool> {}

impl HasAutomaticFromPyObject<true> {
    #[deprecated(
        since = "0.28.0",
        note = "The automatically derived `FromPyObject` implementation for `#[pyclass]` types which implement `Clone` is being phased out. Use `from_py_object` to keep the automatic derive or `skip_from_py_object` to accept the new behaviour."
    )]
    pub const MSG: () = ();
}

impl HasAutomaticFromPyObject<false> {
    pub const MSG: () = ();
}
