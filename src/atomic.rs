use std::{
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{ffi, Bound, Py, Python};

/// Variant of [`Py<T>`] that can be atomically swapped
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicPy<T> {
    inner: AtomicPtr<ffi::PyObject>,
    _marker: PhantomData<T>,
}

// SAFETY: same as `Py<T>`
unsafe impl<T> Send for AtomicPy<T> {}
unsafe impl<T> Sync for AtomicPy<T> {}

impl<T> AtomicPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Py<T>`]
    #[inline]
    pub fn from_py(obj: Py<T>) -> Self {
        Self {
            inner: AtomicPtr::new(obj.into_ptr()),
            _marker: PhantomData,
        }
    }

    /// Create an [`AtomicPy<T>`] holding the [`Bound<T>`]
    #[inline]
    pub fn from_bound(obj: Bound<'_, T>) -> Self {
        Self::from_py(obj.unbind())
    }

    /// Consumes the [`AtomicPy<T>`] and turns it back into a [`Py<T>`]
    #[inline]
    pub fn into_inner(self) -> Py<T> {
        let ptr = *std::mem::ManuallyDrop::new(self).inner.get_mut();
        // SAFETY: `ptr` is a non-null, owned pointer to `ffi::PyObject` of type `T`
        unsafe { Py::from_owned_ptr_unchecked(ptr) }
    }

    /// Stores `obj` in the [`AtomicPy`], returning the previous value.
    ///
    /// See [`swap`](AtomicPy::swap)
    #[inline]
    pub fn swap_unbound(&self, obj: Py<T>, order: Ordering) -> Py<T> {
        let ptr = self.inner.swap(obj.into_ptr(), order);
        // SAFETY: `ptr` is a non-null, owned pointer to `ffi::PyObject` of type `T`
        unsafe { Py::from_owned_ptr_unchecked(ptr) }
    }

    /// Stores `obj` in the [`AtomicPy`], returning the previous value.
    ///
    /// `swap` takes an [Ordering] argument which describes the memory ordering of this operation.
    /// All ordering modes are possible. Note that using [Acquire](Ordering::Acquire) makes the
    /// store part of this operation [Relaxed](Ordering::Relaxed), and using
    /// [Release](Ordering::Release) makes the load part [Relaxed](Ordering::Relaxed).
    #[inline]
    pub fn swap<'py>(&self, obj: Bound<'py, T>, order: Ordering) -> Bound<'py, T> {
        let py = obj.py();
        self.swap_unbound(obj.unbind(), order).into_bound(py)
    }
}

impl<T> From<Py<T>> for AtomicPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Py<T>`]
    fn from(obj: Py<T>) -> Self {
        Self::from_py(obj)
    }
}

impl<T> From<Bound<'_, T>> for AtomicPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Bound<T>`]
    fn from(obj: Bound<'_, T>) -> Self {
        Self::from_bound(obj)
    }
}

impl<T> Drop for AtomicPy<T> {
    /// Drop the inner [`Py<T>`]
    fn drop(&mut self) {
        // SAFETY: `inner` is a non-null, owned pointer to `ffi::PyObject` of type `T`
        unsafe { Py::<T>::from_owned_ptr_unchecked(*self.inner.get_mut()) };
    }
}

/// Variant of [`Option<Py<T>>`] that can be atomically swapped
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicOptionPy<T> {
    inner: AtomicPtr<ffi::PyObject>,
    _marker: PhantomData<T>,
}

// SAFETY: same as `Py<T>`
unsafe impl<T> Send for AtomicOptionPy<T> {}
unsafe impl<T> Sync for AtomicOptionPy<T> {}

impl<T> AtomicOptionPy<T> {
    /// Create an [`AtomicOptionPy<T>`] holding the [`Py<T>`]
    #[inline]
    pub fn from_py(obj: Py<T>) -> Self {
        Self {
            inner: AtomicPtr::new(obj.into_ptr()),
            _marker: PhantomData,
        }
    }

    /// Create an [`AtomicOptionPy<T>`] holding the [`Bound<T>`]
    #[inline]
    pub fn from_bound(obj: Bound<'_, T>) -> Self {
        Self::from_py(obj.unbind())
    }

    /// Consumes the [`AtomicOptionPy<T>`] and turns it back into a [`Py<T>`] or [`None`] if empty
    #[inline]
    pub fn into_inner(self) -> Option<Py<T>> {
        let ptr = *std::mem::ManuallyDrop::new(self).inner.get_mut();
        // SAFETY: `ptr` is a owned pointer to `ffi::PyObject` of type `T`
        NonNull::new(ptr).map(|ptr| unsafe { Py::from_owned_non_null(ptr) })
    }

    /// Takes the object out of the [`AtomicOptionPy`], leaving [`None`] in its place
    ///
    /// Note: This uses [`swap`](Self::swap) under the hood.
    #[inline]
    pub fn take<'py>(&self, py: Python<'py>, order: Ordering) -> Option<Bound<'py, T>> {
        self.swap(py, None, order)
    }

    /// Stores `obj` in the [`AtomicPy`], returning the previous value.
    ///
    /// See [`swap`](AtomicPy::swap)
    #[inline]
    pub fn swap_unbound(&self, obj: Option<Py<T>>, order: Ordering) -> Option<Py<T>> {
        let ptr = self
            .inner
            .swap(obj.map(Py::into_ptr).unwrap_or_default(), order);
        // SAFETY: `ptr` is an owned pointer to `ffi::PyObject` of type `T`
        NonNull::new(ptr).map(|ptr| unsafe { Py::from_owned_non_null(ptr) })
    }

    /// Stores `obj` in the [`AtomicPy`], returning the previous value.
    ///
    /// `swap` takes an [Ordering] argument which describes the memory ordering of this operation.
    /// All ordering modes are possible. Note that using [Acquire](Ordering::Acquire) makes the
    /// store part of this operation [Relaxed](Ordering::Relaxed), and using
    /// [Release](Ordering::Release) makes the load part [Relaxed](Ordering::Relaxed).
    #[inline]
    pub fn swap<'py>(
        &self,
        py: Python<'py>,
        obj: Option<Bound<'py, T>>,
        order: Ordering,
    ) -> Option<Bound<'py, T>> {
        self.swap_unbound(obj.map(Bound::unbind), order)
            .map(|obj| obj.into_bound(py))
    }
}

impl<T> Default for AtomicOptionPy<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<T> From<Py<T>> for AtomicOptionPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Py<T>`]
    fn from(obj: Py<T>) -> Self {
        Self::from_py(obj)
    }
}

impl<T> From<Bound<'_, T>> for AtomicOptionPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Bound<T>`]
    fn from(obj: Bound<'_, T>) -> Self {
        Self::from_bound(obj)
    }
}

impl<T> From<Option<Py<T>>> for AtomicOptionPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Py<T>`]
    fn from(obj: Option<Py<T>>) -> Self {
        if let Some(obj) = obj {
            Self::from_py(obj)
        } else {
            Self::default()
        }
    }
}

impl<T> From<Option<Bound<'_, T>>> for AtomicOptionPy<T> {
    /// Create an [`AtomicPy<T>`] holding the [`Bound<T>`]
    fn from(obj: Option<Bound<'_, T>>) -> Self {
        if let Some(obj) = obj {
            Self::from_bound(obj)
        } else {
            Self::default()
        }
    }
}

impl<T> Drop for AtomicOptionPy<T> {
    /// Drop the inner [`Py<T>`]
    fn drop(&mut self) {
        if let Some(ptr) = NonNull::new(*self.inner.get_mut()) {
            // SAFETY: `ptr` is an owned pointer to `ffi::PyObject` of type `T`
            unsafe { Py::<T>::from_owned_non_null(ptr) };
        }
    }
}

#[cfg(test)]
mod tests {}
