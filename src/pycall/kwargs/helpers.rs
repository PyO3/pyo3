use crate::err::error_on_minusone;
use crate::types::{PyDict, PyString};
use crate::{ffi, Borrowed, BoundObject, PyAny, PyErr, PyResult, Python};
use std::marker::PhantomData;

use crate::conversion::IntoPyObject;
use crate::pycall::PPPyObject;
use crate::types::PyTuple;

use super::{ConcatArrays, ExistingNames};

pub struct DropOneGuard<'py, DropTy> {
    ptr: PPPyObject,
    py: Python<'py>,
    _marker: PhantomData<DropTy>,
}
impl<'py, DropTy> DropOneGuard<'py, DropTy> {
    #[inline(always)]
    pub(super) fn from_write<K, V>(
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
        name: K,
        value: V,
    ) -> PyResult<Self>
    where
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py, Output = DropTy>,
        DropTy: BoundObject<'py, V::Target>,
    {
        const {
            assert!(
                size_of::<*mut ffi::PyObject>() == size_of::<DropTy>()
                    && align_of::<*mut ffi::PyObject>() == align_of::<DropTy>(),
            )
        }

        let py = kwargs_tuple.py();
        let name = name.into_pyobject(py).map_err(Into::into)?.into_bound();
        let value = value.into_pyobject(py).map_err(Into::into)?;
        existing_names.insert(
            name,
            value.as_borrowed().into_any(),
            args,
            kwargs_tuple,
            index,
        )?;
        Ok(Self {
            ptr: args,
            py,
            _marker: PhantomData,
        })
    }
    #[inline(always)]
    pub(super) fn write<K, V, NextDropTy>(
        self,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
        name: K,
        value: V,
    ) -> PyResult<DropOneGuard<'py, ConcatArrays<DropTy, NextDropTy>>>
    where
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py, Output = NextDropTy>,
        NextDropTy: BoundObject<'py, V::Target>,
    {
        let py = kwargs_tuple.py();
        let name = name.into_pyobject(py).map_err(Into::into)?.into_bound();
        let value = value.into_pyobject(py).map_err(Into::into)?;
        existing_names.insert(
            name,
            value.as_borrowed().into_any(),
            self.ptr,
            kwargs_tuple,
            index,
        )?;
        Ok(DropOneGuard {
            ptr: self.ptr,
            py: self.py,
            _marker: PhantomData,
        })
    }
}
impl<DropTy> Drop for DropOneGuard<'_, DropTy> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.ptr.cast::<DropTy>().drop_in_place();
        }
    }
}

pub struct DropManyGuard<DropTy> {
    ptr: PPPyObject,
    len: usize,
    _marker: PhantomData<DropTy>,
}
impl<DropTy> DropManyGuard<DropTy> {
    #[inline(always)]
    pub(super) fn from_iter<'py, K, V>(
        args: PPPyObject,
        kwargs_tuple: Borrowed<'_, 'py, PyTuple>,
        iter: impl IntoIterator<Item = (K, V)>,
        index: &mut ffi::Py_ssize_t,
        existing_names: &mut ExistingNames,
    ) -> PyResult<Self>
    where
        K: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py, Output = DropTy>,
        DropTy: BoundObject<'py, V::Target>,
    {
        let mut guard = Self {
            ptr: args,
            len: 0,
            _marker: PhantomData,
        };
        let py = kwargs_tuple.py();
        iter.into_iter().try_for_each(|(name, value)| {
            let name = name.into_pyobject(py).map_err(Into::into)?.into_bound();
            let value = value.into_pyobject(py).map_err(Into::into)?;
            existing_names.insert(
                name,
                value.as_borrowed().into_any(),
                args,
                kwargs_tuple,
                index,
            )?;
            guard.len += 1;
            Ok::<_, PyErr>(())
        })?;
        Ok(guard)
    }
}
impl<DropTy> Drop for DropManyGuard<DropTy> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            std::ptr::slice_from_raw_parts_mut(self.ptr.cast::<DropTy>(), self.len).drop_in_place();
        }
    }
}

#[inline(always)]
pub(super) fn set_kwarg(
    dict: Borrowed<'_, '_, PyDict>,
    name: Borrowed<'_, '_, PyString>,
    value: Borrowed<'_, '_, PyAny>,
) -> PyResult<()> {
    unsafe {
        error_on_minusone(
            dict.py(),
            ffi::PyDict_SetItem(dict.as_ptr(), name.as_ptr(), value.as_ptr()),
        )
    }
}

#[inline(always)]
pub(super) fn set_kwargs_from_iter<'py, K, V>(
    dict: Borrowed<'_, 'py, PyDict>,
    iter: impl IntoIterator<Item = (K, V)>,
) -> PyResult<()>
where
    K: IntoPyObject<'py, Target = PyString>,
    V: IntoPyObject<'py>,
{
    let py = dict.py();
    for (name, value) in iter {
        set_kwarg(
            dict,
            name.into_pyobject(py).map_err(Into::into)?.as_borrowed(),
            value
                .into_pyobject(py)
                .map_err(Into::into)?
                .into_any()
                .as_borrowed(),
        )?;
    }
    Ok(())
}
