use std::marker::PhantomData;

use crate::conversion::IntoPyObject;
use crate::pycall::storage::{DynKnownSizeRawStorage, RawStorage};
use crate::pycall::trusted_len::TrustedLen;
use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, BoundObject, PyResult, Python};

use super::helpers::{
    concat_known_sized, write_iter_to_tuple, write_raw_storage_to_tuple, DropManyGuard,
};
use super::{ArgumentsOffsetFlag, ConcatStorages, ResolveArgs};

pub struct VecArgsStorage<T>(pub(in super::super) T);

impl<'py, T> ResolveArgs<'py> for VecArgsStorage<T>
where
    T: ResolveArgs<'py>,
    T::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
{
    type RawStorage = T::RawStorage;
    type Guard = T::Guard;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        self.0.init(py, storage, base_storage)
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
        T::write_initialized_to_tuple(tuple, guard, raw_storage, index)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    #[inline(always)]
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        self.0.as_pytuple(py)
    }
    const IS_EMPTY: bool = T::IS_EMPTY;
    const IS_ONE: bool = T::IS_ONE;
    const USE_STACK_FOR_SMALL_LEN: bool = T::USE_STACK_FOR_SMALL_LEN;
}

impl<'py, A, B> ResolveArgs<'py> for VecArgsStorage<ConcatStorages<A, B>>
where
    A: ResolveArgs<'py>,
    B: ResolveArgs<'py>,
    A::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
    B::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
{
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = (A::Guard, B::Guard);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        concat_known_sized(self.0 .0, self.0 .1, py, storage, base_storage)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0 .0.len() + self.0 .1.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        self.0 .0.write_to_tuple(tuple, index)?;
        self.0 .1.write_to_tuple(tuple, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        A::write_initialized_to_tuple(tuple, guard.0, raw_storage, index);
        B::write_initialized_to_tuple(tuple, guard.1, raw_storage, index);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        true
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;
    const IS_ONE: bool = (A::IS_EMPTY && B::IS_ONE) || (A::IS_ONE && B::IS_EMPTY);
    const USE_STACK_FOR_SMALL_LEN: bool = true;
}

pub struct TrustedLenIterator<I>(pub(super) I);

impl<'py, I, Item> ResolveArgs<'py> for TrustedLenIterator<I>
where
    I: TrustedLen<Item = Item>,
    Item: IntoPyObject<'py>,
{
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = DropManyGuard<Item::Output>;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        DropManyGuard::from_iter(py, storage, base_storage, self.0)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.size_hint().0
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        write_iter_to_tuple(tuple, self.0, index)
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<Item::Output, _>(tuple, raw_storage, index, guard.len())
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

pub struct ExactSizeIterator<I> {
    iter: I,
    // We cannot rely on the iterator providing the same `len()` every time (this will lead to unsoundness),
    // so we save it here.
    len: usize,
}

impl<I: std::iter::ExactSizeIterator> ExactSizeIterator<I> {
    #[inline(always)]
    pub(super) fn new(iter: I) -> Self {
        Self {
            len: iter.len(),
            iter,
        }
    }
}

impl<'py, I, Item> ResolveArgs<'py> for ExactSizeIterator<I>
where
    I: std::iter::ExactSizeIterator<Item = Item>,
    Item: IntoPyObject<'py>,
{
    type RawStorage = DynKnownSizeRawStorage;
    type Guard = DropManyGuard<Item::Output>;
    #[inline(always)]
    fn init(
        mut self,
        py: Python<'py>,
        storage: PPPyObject,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        struct Guard<T>(PPPyObject, usize, PhantomData<T>);
        impl<T> Drop for Guard<T> {
            #[inline(always)]
            fn drop(&mut self) {
                unsafe {
                    std::ptr::slice_from_raw_parts_mut(self.0, self.1).drop_in_place();
                }
            }
        }
        unsafe {
            let len = self.len;
            let mut guard = Guard::<Item::Output>(storage, 0, PhantomData);
            self.iter.try_for_each(|item| {
                if guard.1 >= len {
                    // FIXME: Maybe this should be an `Err` and not panic?
                    panic!("an ExactSizeIterator produced more items than it declared");
                }
                match item.into_pyobject(py) {
                    Ok(item) => {
                        guard.0.add(guard.1).write(item.into_ptr_raw());
                        guard.1 += 1;
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            })?;
            if guard.1 != len {
                panic!("an ExactSizeIterator produced less items than it declared");
            }
            Ok(DropManyGuard::new(base_storage, guard.0, guard.1))
        }
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn write_to_tuple(
        mut self,
        tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        let mut i = *index;
        let len = self.len as isize;
        unsafe {
            self.iter.try_for_each(|item| {
                if i >= len {
                    // FIXME: Maybe this should be an `Err` and not panic?
                    panic!("an ExactSizeIterator produced more items than it declared");
                }
                match item.into_pyobject(tuple.py()) {
                    Ok(item) => {
                        ffi::PyTuple_SET_ITEM(tuple.as_ptr(), i, item.into_ptr_raw());
                        i += 1;
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            })?;
        }
        if i != len {
            panic!("an ExactSizeIterator produced less items than it declared");
        }
        *index = i;
        Ok(())
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<Item::Output, _>(tuple, raw_storage, index, guard.len())
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
