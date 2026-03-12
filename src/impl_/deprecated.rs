pub struct HasAutomaticFromPyObject<const IS_CLONE: bool> {}

impl HasAutomaticFromPyObject<true> {
    #[deprecated(
        since = "0.28.0",
        note = "The `FromPyObject` implementation for `#[pyclass]` types which implement `Clone` is changing to an opt-in option. Use `#[pyclass(from_py_object)]` to opt-in to the `FromPyObject` derive now, or `#[pyclass(skip_from_py_object)]` to skip the `FromPyObject` implementation."
    )]
    pub const MSG: () = ();
}

impl HasAutomaticFromPyObject<false> {
    pub const MSG: () = ();
}
