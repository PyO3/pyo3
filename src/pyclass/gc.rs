use core::{
    ffi::CStr,
    ffi::{c_int, c_void},
    marker::PhantomData,
    num::NonZero,
    ops::{Deref, DerefMut},
};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    ffi::CString,
    string::String,
    vec::Vec,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    hash::{BuildHasher, Hash},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::{ffi, Py};

/// Trait describing how values participate in Python's cyclic garbage collector.
///
/// # Safety
///
/// Implementations must not execute arbitrary Python code from `traverse`.
pub unsafe trait PyGcTraversable {
    /// Whether this type may hold Python object references which can participate in cycles.
    const MAY_CONTAIN_CYCLES: bool;

    /// Visit all Python objects referenced by this value.
    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>;

    /// Clear references held by this value to help break cycles.
    fn clear(&mut self);
}

macro_rules! impl_py_gc_no_cycles {
    ($($ty:ty),* $(,)?) => {
        $(
            unsafe impl PyGcTraversable for $ty {
                const MAY_CONTAIN_CYCLES: bool = false;

                #[inline]
                fn traverse(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
                    Ok(())
                }

                #[inline]
                fn clear(&mut self) {}
            }
        )*
    };
}

impl_py_gc_no_cycles!(
    (),
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    str,
    String,
    CString,
    CStr,
    Path,
    PathBuf,
    OsStr,
    OsString,
);

unsafe impl<T: ?Sized + PyGcTraversable> PyGcTraversable for &T {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    #[inline]
    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            (**self).traverse(visit)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn clear(&mut self) {}
}

unsafe impl<T: ?Sized + PyGcTraversable> PyGcTraversable for &mut T {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    #[inline]
    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            (**self).traverse(visit)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            (**self).clear();
        }
    }
}

unsafe impl<T> PyGcTraversable for PhantomData<T> {
    const MAY_CONTAIN_CYCLES: bool = false;

    #[inline]
    fn traverse(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {}
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for Option<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            if let Some(value) = self {
                value.traverse(visit)?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            *self = None;
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for Vec<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            Vec::clear(self);
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for VecDeque<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            VecDeque::clear(self);
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for LinkedList<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            LinkedList::clear(self);
        }
    }
}

unsafe impl<T: PyGcTraversable + Ord> PyGcTraversable for BinaryHeap<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            BinaryHeap::clear(self);
        }
    }
}

unsafe impl<K, V, S> PyGcTraversable for HashMap<K, V, S>
where
    K: PyGcTraversable + Eq + Hash,
    V: PyGcTraversable,
    S: BuildHasher,
{
    const MAY_CONTAIN_CYCLES: bool = K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES {
            for (key, value) in self {
                if K::MAY_CONTAIN_CYCLES {
                    key.traverse(visit.clone())?;
                }
                if V::MAY_CONTAIN_CYCLES {
                    value.traverse(visit.clone())?;
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES {
            HashMap::clear(self);
        }
    }
}

unsafe impl<T, S> PyGcTraversable for HashSet<T, S>
where
    T: PyGcTraversable + Eq + Hash,
    S: BuildHasher,
{
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            HashSet::clear(self);
        }
    }
}

unsafe impl<K, V> PyGcTraversable for BTreeMap<K, V>
where
    K: PyGcTraversable + Ord,
    V: PyGcTraversable,
{
    const MAY_CONTAIN_CYCLES: bool = K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES {
            for (key, value) in self {
                if K::MAY_CONTAIN_CYCLES {
                    key.traverse(visit.clone())?;
                }
                if V::MAY_CONTAIN_CYCLES {
                    value.traverse(visit.clone())?;
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if K::MAY_CONTAIN_CYCLES || V::MAY_CONTAIN_CYCLES {
            BTreeMap::clear(self);
        }
    }
}

unsafe impl<T: PyGcTraversable + Ord> PyGcTraversable for BTreeSet<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            BTreeSet::clear(self);
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for OnceLock<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            if let Some(value) = self.get() {
                value.traverse(visit)?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            let _ = self.take();
        }
    }
}

unsafe impl<T, E> PyGcTraversable for Result<T, E>
where
    T: PyGcTraversable,
    E: PyGcTraversable,
{
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES || E::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES || E::MAY_CONTAIN_CYCLES {
            match self {
                Ok(value) if T::MAY_CONTAIN_CYCLES => value.traverse(visit)?,
                Err(error) if E::MAY_CONTAIN_CYCLES => error.traverse(visit)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES || E::MAY_CONTAIN_CYCLES {
            match self {
                Ok(value) if T::MAY_CONTAIN_CYCLES => value.clear(),
                Err(error) if E::MAY_CONTAIN_CYCLES => error.clear(),
                _ => {}
            }
        }
    }
}

unsafe impl<T: PyGcTraversable, const N: usize> PyGcTraversable for [T; N] {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.clear();
            }
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for [T] {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.traverse(visit.clone())?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            for item in self {
                item.clear();
            }
        }
    }
}

unsafe impl<T: PyGcTraversable> PyGcTraversable for Box<T> {
    const MAY_CONTAIN_CYCLES: bool = T::MAY_CONTAIN_CYCLES;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if T::MAY_CONTAIN_CYCLES {
            (**self).traverse(visit)?;
        }
        Ok(())
    }

    fn clear(&mut self) {
        if T::MAY_CONTAIN_CYCLES {
            (**self).clear();
        }
    }
}

unsafe impl<T> PyGcTraversable for Py<T> {
    const MAY_CONTAIN_CYCLES: bool = true;

    fn traverse(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(self)
    }

    fn clear(&mut self) {}
}

/// Wrapper to explicitly opt out of GC traversal for a type.
///
/// This is useful for intentional recursion breakpoints where traversing a
/// reference would recurse indefinitely. Only use this when the wrapped value
/// is known to be traversed through another path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PyGcOpaque<T>(T);

impl<T> PyGcOpaque<T> {
    /// Wrap `inner` as GC-opaque.
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Consume this wrapper and return the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for PyGcOpaque<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for PyGcOpaque<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for PyGcOpaque<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

unsafe impl<T> PyGcTraversable for PyGcOpaque<T> {
    const MAY_CONTAIN_CYCLES: bool = false;

    #[inline]
    fn traverse(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {}
}

/// Error returned by a `__traverse__` visitor implementation.
#[repr(transparent)]
pub struct PyTraverseError(NonZero<c_int>);

impl PyTraverseError {
    /// Returns the error code.
    pub(crate) fn into_inner(self) -> c_int {
        self.0.into()
    }
}

/// Object visitor for GC.
#[derive(Clone)]
pub struct PyVisit<'a> {
    visit: ffi::visitproc,
    arg: *mut c_void,
    /// Prevents the `PyVisit` from outliving the `__traverse__` call.
    _guard: PhantomData<&'a ()>,
}

impl PyVisit<'_> {
    /// Visit `obj`.
    ///
    /// Note: `obj` accepts a variety of types, including
    /// - `&Py<T>`
    /// - `&Option<Py<T>>`
    /// - `Option<&Py<T>>`
    pub fn call<'a, T, U: 'a>(&self, obj: T) -> Result<(), PyTraverseError>
    where
        T: Into<Option<&'a Py<U>>>,
    {
        let ptr = obj.into().map_or_else(core::ptr::null_mut, Py::as_ptr);
        if !ptr.is_null() {
            // SAFETY: `PyVisit` is only created during a `tp_traverse` call and
            // cannot outlive it, so `visit` and `arg` are still the pair the
            // interpreter passed in; `ptr` comes from `Py::as_ptr`, so it is a
            // valid object pointer.
            let retval = unsafe { (self.visit)(ptr, self.arg) };
            make_traverse_result(retval)
        } else {
            Ok(())
        }
    }

    /// # Safety
    ///
    /// The caller must pass a `visit` and `arg` pair that are guaranteed to be valid for
    /// the inferred lifetime `'a`.
    pub(crate) unsafe fn new(visit: ffi::visitproc, arg: *mut c_void) -> Self {
        Self {
            visit,
            arg,
            _guard: PhantomData,
        }
    }
}

/// Cast the return value of a `visitproc` to a `Result<(), PyTraverseError>`.
///
/// This should optimize to a no-op in release builds due to the 0 niche in `PyTraverseError`.
#[inline]
pub(crate) fn make_traverse_result(retval: c_int) -> Result<(), PyTraverseError> {
    match NonZero::new(retval) {
        None => Ok(()),
        Some(r) => Err(PyTraverseError(r)),
    }
}

#[cfg(test)]
mod tests {
    use super::PyVisit;
    use static_assertions::assert_not_impl_any;

    #[test]
    fn py_visit_not_send_sync() {
        assert_not_impl_any!(PyVisit<'_>: Send, Sync);
    }
}
