use crate::{Borrowed, Bound, Py, Python};

pub trait AsPyObject<'py> {
    type PyObject;
    fn as_borrowed(&self, py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject>;
    const IS_OWNED: bool;
    fn into_bound(self, py: Python<'py>) -> Bound<'py, Self::PyObject>;
}

impl<'py, T> AsPyObject<'py> for Bound<'py, T> {
    type PyObject = T;
    #[inline(always)]
    fn as_borrowed(&self, _py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject> {
        self.as_borrowed()
    }
    const IS_OWNED: bool = true;
    #[inline(always)]
    fn into_bound(self, _py: Python<'py>) -> Bound<'py, Self::PyObject> {
        self
    }
}

impl<'py, T> AsPyObject<'py> for Borrowed<'_, 'py, T> {
    type PyObject = T;
    #[inline(always)]
    fn as_borrowed(&self, _py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject> {
        *self
    }
    const IS_OWNED: bool = false;
    #[inline(always)]
    fn into_bound(self, _py: Python<'py>) -> Bound<'py, Self::PyObject> {
        panic!("non-owned AsPyObject cannot be converted into Bound")
    }
}

impl<'py, T> AsPyObject<'py> for Py<T> {
    type PyObject = T;
    #[inline(always)]
    fn as_borrowed(&self, py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject> {
        self.bind_borrowed(py)
    }
    const IS_OWNED: bool = true;
    #[inline(always)]
    fn into_bound(self, py: Python<'py>) -> Bound<'py, Self::PyObject> {
        self.into_bound(py)
    }
}

impl<'py, T: AsPyObject<'py>> AsPyObject<'py> for &'_ T {
    type PyObject = T::PyObject;
    #[inline(always)]
    fn as_borrowed(&self, py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject> {
        T::as_borrowed(*self, py)
    }
    const IS_OWNED: bool = false;
    #[inline(always)]
    fn into_bound(self, _py: Python<'py>) -> Bound<'py, Self::PyObject> {
        panic!("non-owned AsPyObject cannot be converted into Bound")
    }
}

impl<'py, T: AsPyObject<'py>> AsPyObject<'py> for &'_ mut T {
    type PyObject = T::PyObject;
    #[inline(always)]
    fn as_borrowed(&self, py: Python<'py>) -> Borrowed<'_, 'py, Self::PyObject> {
        T::as_borrowed(*self, py)
    }
    const IS_OWNED: bool = false;
    #[inline(always)]
    fn into_bound(self, _py: Python<'py>) -> Bound<'py, Self::PyObject> {
        panic!("non-owned AsPyObject cannot be converted into Bound")
    }
}
