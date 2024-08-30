use std::mem::{ManuallyDrop, MaybeUninit};

use super::helpers::write_raw_storage_to_tuple;
use super::{ArgumentsOffsetFlag, ResolveArgs};
use crate::pycall::storage::{DynKnownSizeRawStorage, RawStorage};
use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, Bound, Py, PyAny, PyResult, Python};

pub struct ExistingArgListSlice<S>(pub(super) S);

pub trait ExistingArgListSliceArg {
    const IS_OWNED: bool;
}
impl<T> ExistingArgListSliceArg for Py<T> {
    const IS_OWNED: bool = true;
}
impl<T> ExistingArgListSliceArg for Bound<'_, T> {
    const IS_OWNED: bool = true;
}
impl<T> ExistingArgListSliceArg for Borrowed<'_, '_, T> {
    const IS_OWNED: bool = false;
}

pub trait ExistingArgListSliceTrait: Sized {
    fn as_ptr(&mut self) -> PPPyObject;
    fn len(&self) -> usize;
    /// Deallocate the memory but do not drop the objects inside.
    #[inline(always)]
    fn dealloc(&mut self) {}
    const IS_OWNED: bool;
    const CAN_MUTATE: bool;
}
impl<T: ExistingArgListSliceArg> ExistingArgListSliceTrait for Vec<T> {
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        self.as_mut_ptr().cast::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn dealloc(&mut self) {
        unsafe {
            Vec::<Borrowed<'_, '_, PyAny>>::from_raw_parts(
                self.as_mut_ptr().cast::<Borrowed<'_, '_, PyAny>>(),
                self.len(),
                self.capacity(),
            );
        }
    }
    const IS_OWNED: bool = T::IS_OWNED;
    const CAN_MUTATE: bool = true;
}
impl<T: ExistingArgListSliceArg> ExistingArgListSliceTrait for &'_ Vec<T> {
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        (**self).as_ptr().cast::<*mut ffi::PyObject>().cast_mut()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }
    const IS_OWNED: bool = false;
    const CAN_MUTATE: bool = false;
}
impl<T: ExistingArgListSliceArg> ExistingArgListSliceTrait for &'_ mut Vec<T> {
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        (**self).as_mut_ptr().cast::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }
    const IS_OWNED: bool = false;
    const CAN_MUTATE: bool = true;
}
impl<T: ExistingArgListSliceArg> ExistingArgListSliceTrait for &'_ [T] {
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        (**self).as_ptr().cast::<*mut ffi::PyObject>().cast_mut()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }
    const IS_OWNED: bool = false;
    const CAN_MUTATE: bool = false;
}
impl<T: ExistingArgListSliceArg> ExistingArgListSliceTrait for &'_ mut [T] {
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        (**self).as_mut_ptr().cast::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }
    const IS_OWNED: bool = false;
    const CAN_MUTATE: bool = true;
}

pub struct ExistingArgListSliceStorage<S>(MaybeUninit<S>);
impl<S: ExistingArgListSliceTrait> RawStorage for ExistingArgListSliceStorage<S> {
    type InitParam<'a> = &'a mut Self
    where
        Self: 'a;
    #[inline(always)]
    fn new(_len: usize) -> Self {
        Self(MaybeUninit::uninit())
    }
    #[inline(always)]
    fn as_init_param(&mut self) -> Self::InitParam<'_> {
        self
    }
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        unsafe { self.0.assume_init_mut().as_ptr() }
    }
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { self.0.assume_init_ref().len() }
    }
    #[inline(always)]
    fn init_param_from_ptr<'a>(_ptr: PPPyObject) -> Self::InitParam<'a> {
        unreachable!("ExistingArgListSliceStorage does not use small stack optimization")
    }
}
impl<S> Drop for ExistingArgListSliceStorage<S> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.0.assume_init_drop();
        }
    }
}
impl<'py, S: ExistingArgListSliceTrait> ResolveArgs<'py> for ExistingArgListSlice<S> {
    type RawStorage = ExistingArgListSliceStorage<S>;
    type Guard = ();
    #[inline(always)]
    fn init(
        self,
        _py: Python<'py>,
        storage: &mut ExistingArgListSliceStorage<S>,
        _base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        storage.0 = MaybeUninit::new(self.0);
        Ok(())
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        let mut this = ManuallyDrop::new(self);
        unsafe {
            let mut p = this.0.as_ptr();
            for i in *index..*index + this.len() as ffi::Py_ssize_t {
                let v = *p;
                if !S::IS_OWNED {
                    ffi::Py_INCREF(v);
                }
                ffi::PyTuple_SET_ITEM(tuple.as_ptr(), i, v);
                p = p.add(1);
            }
        }
        *index += this.len() as isize;
        this.0.dealloc();
        Ok(())
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _guard: Self::Guard,
        _raw_storage: &mut PPPyObject,
        _index: &mut ffi::Py_ssize_t,
    ) {
        unreachable!(
            "ExistingArgListSlice::write_initialized_to_tuple() should never be called: \
            `write_to_tuple()` will be called if the storage is alone, and if it is concatenated, \
            it will be replaced by ExistingArgListVecStorageAdapter"
        )
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = if S::CAN_MUTATE {
        ArgumentsOffsetFlag::DoNotOffsetButCanChangeArgs0
    } else {
        ArgumentsOffsetFlag::DoNotOffset
    };
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

pub struct ExistingArgListVecStorageAdapter<S>(pub(super) ExistingArgListSlice<S>);
impl<'py, S: ExistingArgListSliceTrait> ResolveArgs<'py> for ExistingArgListVecStorageAdapter<S> {
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = S;
    #[inline(always)]
    fn init(
        mut self,
        _py: Python<'py>,
        storage: PPPyObject,
        _base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        unsafe {
            std::ptr::copy_nonoverlapping(self.0 .0.as_ptr(), storage, self.0.len());
        }
        Ok(self.0 .0)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.0.write_to_tuple(tuple, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        let mut guard = ManuallyDrop::new(guard);
        write_raw_storage_to_tuple::<Bound<'_, PyAny>, _>(tuple, raw_storage, index, guard.len());
        guard.dealloc();
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}
