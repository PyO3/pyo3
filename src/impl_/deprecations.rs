//! Symbols used to denote deprecated usages of PyO3's proc macros.

use crate::Python;

#[deprecated(since = "0.20.0", note = "use `#[new]` instead of `#[__new__]`")]
pub const PYMETHODS_NEW_DEPRECATED_FORM: () = ();

pub fn inspect_type<T>(t: T) -> (T, GilRefs<T>) {
    (t, GilRefs::new())
}

pub struct GilRefs<T>(NotAGilRef<T>);
pub struct NotAGilRef<T>(std::marker::PhantomData<T>);

pub trait IsGilRef {}

impl<T: crate::PyNativeType> IsGilRef for &'_ T {}

impl<T> GilRefs<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GilRefs(NotAGilRef(std::marker::PhantomData))
    }
}

impl GilRefs<Python<'_>> {
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(since = "0.21.0", note = "use `wrap_pyfunction_bound!` instead")
    )]
    pub fn is_python(&self) {}
}

impl<T: IsGilRef> GilRefs<T> {
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "use `&Bound<'_, T>` instead for this function argument"
        )
    )]
    pub fn extract_gil_ref(&self) {}
}

impl<T> NotAGilRef<T> {
    pub fn extract_gil_ref(&self) {}
    pub fn is_python(&self) {}
}

impl<T> std::ops::Deref for GilRefs<T> {
    type Target = NotAGilRef<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
