//! Symbols used to denote deprecated usages of PyO3's proc macros.

use crate::{PyResult, Python};

#[deprecated(since = "0.20.0", note = "use `#[new]` instead of `#[__new__]`")]
pub const PYMETHODS_NEW_DEPRECATED_FORM: () = ();

pub fn inspect_type<T>(t: T, _: &GilRefs<T>) -> T {
    t
}

pub fn inspect_fn<A, T>(f: fn(A) -> PyResult<T>, _: &GilRefs<A>) -> fn(A) -> PyResult<T> {
    f
}

pub struct GilRefs<T>(OptionGilRefs<T>);
pub struct OptionGilRefs<T>(NotAGilRef<T>);
pub struct NotAGilRef<T>(std::marker::PhantomData<T>);

pub trait IsGilRef {}

impl<T: crate::PyNativeType> IsGilRef for &'_ T {}

impl<T> GilRefs<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GilRefs(OptionGilRefs(NotAGilRef(std::marker::PhantomData)))
    }
}

impl GilRefs<Python<'_>> {
    #[deprecated(since = "0.21.0", note = "use `wrap_pyfunction_bound!` instead")]
    pub fn is_python(&self) {}
}

impl<T: IsGilRef> GilRefs<T> {
    #[deprecated(
        since = "0.21.0",
        note = "use `&Bound<'_, T>` instead for this function argument"
    )]
    pub fn function_arg(&self) {}
    #[deprecated(
        since = "0.21.0",
        note = "use `&Bound<'_, PyAny>` as the argument for this `from_py_with` extractor"
    )]
    pub fn from_py_with_arg(&self) {}
}

impl<T: IsGilRef> OptionGilRefs<Option<T>> {
    #[deprecated(
        since = "0.21.0",
        note = "use `Option<&Bound<'_, T>>` instead for this function argument"
    )]
    pub fn function_arg(&self) {}
}

impl<T> NotAGilRef<T> {
    pub fn function_arg(&self) {}
    pub fn from_py_with_arg(&self) {}
    pub fn is_python(&self) {}
}

impl<T> std::ops::Deref for GilRefs<T> {
    type Target = OptionGilRefs<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::Deref for OptionGilRefs<T> {
    type Target = NotAGilRef<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
