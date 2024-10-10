use std::marker::PhantomData;
use std::ops::Range;

use crate::conversion::IntoPyObject;
use crate::pycall::storage::RawStorage;
use crate::pycall::PPPyObject;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, BoundObject, PyErr, PyResult, Python};

use super::{ConcatArrays, ResolveArgs};

pub struct DropOneGuard<'py, DropTy> {
    base_storage: *const PPPyObject,
    base_offset: isize,
    py: Python<'py>,
    _marker: PhantomData<DropTy>,
}
impl<'py, DropTy> DropOneGuard<'py, DropTy> {
    #[inline(always)]
    pub(super) fn from_write<T, E>(
        py: Python<'py>,
        write_at: PPPyObject,
        base_storage: *const PPPyObject,
        value: T,
    ) -> PyResult<Self>
    where
        T: IntoPyObject<'py, Output = DropTy, Error = E>,
        E: Into<PyErr>,
    {
        const {
            assert!(
                size_of::<*mut ffi::PyObject>() == size_of::<DropTy>()
                    && align_of::<*mut ffi::PyObject>() == align_of::<DropTy>(),
            )
        }
        unsafe {
            let base = base_storage.read();
            let base_idx = write_at.offset_from(base);
            let value = value.into_pyobject(py).map_err(Into::into)?;
            base.offset(base_idx).cast::<DropTy>().write(value);
            Ok(Self {
                base_storage,
                base_offset: base_idx,
                py,
                _marker: PhantomData,
            })
        }
    }
    #[inline(always)]
    pub(super) fn write<T, E, NextDropTy>(
        self,
        value: T,
    ) -> PyResult<DropOneGuard<'py, ConcatArrays<DropTy, NextDropTy>>>
    where
        T: IntoPyObject<'py, Output = NextDropTy, Error = E>,
        E: Into<PyErr>,
    {
        const {
            assert!(
                size_of::<*mut ffi::PyObject>() == size_of::<NextDropTy>()
                    && align_of::<*mut ffi::PyObject>() == align_of::<NextDropTy>(),
            )
        }
        unsafe {
            let value = value.into_pyobject(self.py).map_err(Into::into)?;
            self.base_storage
                .read()
                .offset(self.base_offset)
                .cast::<DropTy>()
                .add(1)
                .cast::<NextDropTy>()
                .write(value);
            Ok(DropOneGuard {
                base_storage: self.base_storage,
                base_offset: self.base_offset,
                py: self.py,
                _marker: PhantomData,
            })
        }
    }
}
impl<DropTy> Drop for DropOneGuard<'_, DropTy> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.base_storage
                .read()
                .offset(self.base_offset)
                .cast::<DropTy>()
                .drop_in_place();
        }
    }
}

pub struct DropManyGuard<DropTy> {
    base_storage: *const PPPyObject,
    base_range: Range<isize>,
    _marker: PhantomData<DropTy>,
}
impl<DropTy> DropManyGuard<DropTy> {
    #[inline(always)]
    pub(super) fn new(base_storage: *const PPPyObject, from: PPPyObject, len: usize) -> Self {
        let from = unsafe { from.offset_from(base_storage.read()) };
        Self {
            base_storage,
            base_range: from..len as isize + from,
            _marker: PhantomData,
        }
    }
    #[inline(always)]
    pub(super) fn from_iter<'py, T, E>(
        py: Python<'py>,
        write_at: PPPyObject,
        base_storage: *const PPPyObject,
        iter: impl IntoIterator<Item = T>,
    ) -> PyResult<Self>
    where
        T: IntoPyObject<'py, Output = DropTy, Error = E>,
        E: Into<PyErr>,
    {
        const {
            assert!(
                size_of::<*mut ffi::PyObject>() == size_of::<DropTy>()
                    && align_of::<*mut ffi::PyObject>() == align_of::<DropTy>(),
            )
        }

        unsafe {
            let base_offset = write_at.offset_from(base_storage.read());
            let mut guard = Self {
                base_storage,
                base_range: base_offset..base_offset,
                _marker: PhantomData,
            };
            iter.into_iter()
                .try_for_each(|item| match item.into_pyobject(py) {
                    Ok(item) => {
                        let base = guard.base_storage.read();
                        base.offset(guard.base_range.end)
                            .cast::<DropTy>()
                            .write(item);
                        guard.base_range.end += 1;
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                })?;
            Ok(guard)
        }
    }
    #[inline(always)]
    pub(super) fn len(&self) -> usize {
        self.base_range.end as usize - self.base_range.start as usize
    }
}
impl<DropTy> Drop for DropManyGuard<DropTy> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            std::ptr::slice_from_raw_parts_mut(
                self.base_storage
                    .read()
                    .offset(self.base_range.start)
                    .cast::<DropTy>(),
                (self.base_range.end - self.base_range.start) as usize,
            )
            .drop_in_place();
        }
    }
}

pub(super) struct WriteToTuple<'a, 'py> {
    tuple: *mut ffi::PyObject,
    index: &'a mut ffi::Py_ssize_t,
    py: Python<'py>,
}
impl<'a, 'py> WriteToTuple<'a, 'py> {
    #[inline(always)]
    pub(super) fn new(tuple: Borrowed<'_, 'py, PyTuple>, index: &'a mut ffi::Py_ssize_t) -> Self {
        Self {
            tuple: tuple.as_ptr(),
            index,
            py: tuple.py(),
        }
    }

    #[inline(always)]
    pub(super) fn write<T, E>(self, value: T) -> PyResult<Self>
    where
        T: IntoPyObject<'py, Error = E>,
        E: Into<PyErr>,
    {
        unsafe {
            ffi::PyTuple_SET_ITEM(
                self.tuple,
                *self.index,
                value
                    .into_pyobject(self.py)
                    .map_err(Into::into)?
                    .into_bound()
                    .into_ptr(),
            );
        }
        *self.index += 1;
        Ok(self)
    }

    #[inline(always)]
    pub(super) fn finish(self) -> PyResult<()> {
        Ok(())
    }
}

#[inline(always)]
pub(super) fn write_iter_to_tuple<'py, T, E>(
    tuple: Borrowed<'_, 'py, PyTuple>,
    iter: impl IntoIterator<Item = T>,
    index: &mut ffi::Py_ssize_t,
) -> PyResult<()>
where
    T: IntoPyObject<'py, Error = E>,
    E: Into<PyErr>,
{
    iter.into_iter()
        .try_for_each(|item| match item.into_pyobject(tuple.py()) {
            Ok(item) => {
                let item = item.into_bound().into_ptr();
                unsafe {
                    ffi::PyTuple_SET_ITEM(tuple.as_ptr(), *index, item);
                }
                *index += 1;
                Ok(())
            }
            Err(err) => Err(err.into()),
        })
}

#[inline(always)]
pub(super) fn write_raw_storage_to_tuple<'py, T, U>(
    tuple: Borrowed<'_, 'py, PyTuple>,
    raw_storage: &mut PPPyObject,
    index: &mut ffi::Py_ssize_t,
    len: usize,
) where
    T: BoundObject<'py, U>,
{
    let end_index = *index + len as ffi::Py_ssize_t;
    let mut p = *raw_storage;
    for i in *index..end_index {
        unsafe {
            let value = *p;
            if !T::IS_OWNED {
                ffi::Py_INCREF(value);
            }
            ffi::PyTuple_SET_ITEM(tuple.as_ptr(), i, value);
            p = p.add(1);
        }
    }
    *raw_storage = p;
    *index = end_index;
}

#[inline(always)]
pub(super) fn concat_known_sized<'py, A, B>(
    a: A,
    b: B,
    py: Python<'py>,
    mut storage: PPPyObject,
    base_storage: *const PPPyObject,
) -> PyResult<(A::Guard, B::Guard)>
where
    A: ResolveArgs<'py>,
    B: ResolveArgs<'py>,
    A::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject>,
    B::RawStorage: for<'a> RawStorage<InitParam<'a> = PPPyObject>,
{
    let len1 = a.len();
    let index = unsafe { storage.offset_from(base_storage.read()) as usize };
    let g1 = a.init(py, storage, base_storage)?;
    storage = unsafe { base_storage.read().add(index + len1) };
    let g2 = b.init(py, storage, base_storage)?;
    Ok((g1, g2))
}
