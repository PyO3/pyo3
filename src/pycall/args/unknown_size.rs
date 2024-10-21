use std::marker::PhantomData;

use crate::conversion::IntoPyObject;
use crate::pycall::storage::{RawStorage, UnsizedInitParam, UnsizedStorage};
use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, BoundObject, PyErr, PyResult, Python};

use super::helpers::write_raw_storage_to_tuple;
use super::{ArgumentsOffsetFlag, ConcatStorages, ResolveArgs};

pub struct UnsizedArgsStorage<T>(pub(in super::super) T);

impl<'py, T> ResolveArgs<'py> for UnsizedArgsStorage<T>
where
    T: ResolveArgs<'py>,
    T::RawStorage: for<'a> RawStorage<InitParam<'a> = UnsizedInitParam<'a>> + 'static,
{
    type RawStorage = T::RawStorage;
    type Guard = T::Guard;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
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
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        panic!("unsized storages don't support direct writing into tuples")
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
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        self.0.as_pytuple(py)
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        self.0.has_known_size()
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = T::IS_EMPTY;
    const IS_ONE: bool = T::IS_ONE;
    const USE_STACK_FOR_SMALL_LEN: bool = T::USE_STACK_FOR_SMALL_LEN;
}

impl<'py, A, B> ResolveArgs<'py> for UnsizedArgsStorage<ConcatStorages<A, B>>
where
    A: ResolveArgs<'py>,
    B: ResolveArgs<'py>,
    A::RawStorage: for<'a> RawStorage<InitParam<'a> = UnsizedInitParam<'a>> + 'static,
    B::RawStorage: for<'a> RawStorage<InitParam<'a> = UnsizedInitParam<'a>> + 'static,
{
    type RawStorage = UnsizedStorage;
    type Guard = (A::Guard, B::Guard);
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        let g1 = self.0 .0.init(py, storage, base_storage)?;
        let g2 = self.0 .1.init(py, storage, base_storage)?;
        Ok((g1, g2))
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0 .0.len() + self.0 .1.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        panic!("unsized storages don't support direct writing into tuples")
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
        self.0 .0.has_known_size() && self.0 .1.has_known_size()
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;
    const IS_ONE: bool = (A::IS_EMPTY && B::IS_ONE) || (A::IS_ONE && B::IS_EMPTY);
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

pub struct SizedToUnsizedStorage<T>(pub(in super::super) T);

impl<'py, T> ResolveArgs<'py> for UnsizedArgsStorage<SizedToUnsizedStorage<T>>
where
    T: ResolveArgs<'py>,
    T::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject> + 'static,
{
    type RawStorage = UnsizedStorage;
    type Guard = T::Guard;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        let len = self.0 .0.len();
        storage.reserve(len);
        unsafe {
            // The buffer might've been invalidated.
            base_storage.cast_mut().write(storage.as_mut_ptr());

            // FIXME: If the Vec will resize we'll get use-after-free.
            let write_to = storage.as_mut_ptr().add(storage.len());
            let guard = self.0 .0.init(py, write_to, base_storage)?;
            storage.set_len(storage.len() + len);
            Ok(guard)
        }
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0 .0.len()
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        panic!("unsized storages don't support direct writing into tuples")
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
    #[inline(always)]
    fn as_pytuple(&self, py: Python<'py>) -> Option<Borrowed<'_, 'py, PyTuple>> {
        self.0 .0.as_pytuple(py)
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = T::IS_EMPTY;
    const IS_ONE: bool = T::IS_ONE;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}

pub struct AnyIteratorArgs<I>(pub(super) I);

pub struct UnsizedGuard<T>(*const PPPyObject, usize, usize, PhantomData<T>);
impl<T> UnsizedGuard<T> {
    #[inline(always)]
    pub(super) fn empty(base_storage: *const PPPyObject) -> Self {
        Self(base_storage, 0, 0, PhantomData)
    }
    #[inline(always)]
    pub(super) fn from_range(
        base_storage: *const PPPyObject,
        start: PPPyObject,
        len: usize,
    ) -> Self {
        Self(
            base_storage,
            unsafe { start.offset_from(base_storage.read()) as usize },
            len,
            PhantomData,
        )
    }
    #[inline(always)]
    pub(super) fn from_iter<'py, I, R, S, E>(
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
        size_hint: usize,
        mut iter: I,
    ) -> PyResult<Self>
    where
        I: Iterator<Item = Result<R, E>>,
        R: BoundObject<'py, S>,
        E: Into<PyErr>,
    {
        storage.reserve(size_hint);
        unsafe {
            // The buffer might've been invalidated.
            base_storage.cast_mut().write(storage.as_mut_ptr());
        }
        let mut guard = UnsizedGuard(base_storage, storage.len(), 0, PhantomData);
        iter.try_for_each(|item| match item {
            Ok(item) => {
                storage.push(item.into_ptr_raw());
                unsafe {
                    // The buffer might've been invalidated.
                    base_storage.cast_mut().write(storage.as_mut_ptr())
                }
                guard.2 += 1;
                Ok(())
            }
            Err(err) => Err(err.into()),
        })?;
        Ok(guard)
    }
    #[inline(always)]
    pub(super) fn len(&self) -> usize {
        self.2
    }
}
impl<T> Drop for UnsizedGuard<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            std::ptr::slice_from_raw_parts_mut(self.0.read().add(self.1).cast::<T>(), self.2)
                .drop_in_place();
        }
    }
}

impl<'py, I, Item> ResolveArgs<'py> for AnyIteratorArgs<I>
where
    I: Iterator<Item = Item>,
    Item: IntoPyObject<'py>,
{
    type RawStorage = UnsizedStorage;
    type Guard = UnsizedGuard<Item::Output>;
    #[inline(always)]
    fn init(
        self,
        py: Python<'py>,
        storage: UnsizedInitParam<'_>,
        base_storage: *const PPPyObject,
    ) -> PyResult<Self::Guard> {
        UnsizedGuard::from_iter(
            storage,
            base_storage,
            self.0.size_hint().0,
            self.0.map(|item| item.into_pyobject(py)),
        )
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.size_hint().0
    }
    #[inline(always)]
    fn write_to_tuple(
        self,
        _tuple: Borrowed<'_, 'py, PyTuple>,
        _index: &mut ffi::Py_ssize_t,
    ) -> PyResult<()> {
        panic!("unsized storages don't support direct writing into tuples")
    }
    #[inline(always)]
    fn write_initialized_to_tuple(
        tuple: Borrowed<'_, 'py, PyTuple>,
        guard: Self::Guard,
        raw_storage: &mut PPPyObject,
        index: &mut ffi::Py_ssize_t,
    ) {
        write_raw_storage_to_tuple::<Item::Output, _>(tuple, raw_storage, index, guard.len());
        std::mem::forget(guard);
    }
    #[inline(always)]
    fn has_known_size(&self) -> bool {
        false
    }
    const ARGUMENTS_OFFSET: ArgumentsOffsetFlag = ArgumentsOffsetFlag::Normal;
    const IS_EMPTY: bool = false;
    const IS_ONE: bool = false;
    const USE_STACK_FOR_SMALL_LEN: bool = false;
}
