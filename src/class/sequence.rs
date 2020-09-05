// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use crate::callback::IntoPyCallbackOutput;
use crate::conversion::{FromPyObject, IntoPy};
use crate::err::PyErr;
use crate::pyclass::maybe_push_slot;
use crate::{exceptions, ffi, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::{c_int, c_void};

/// Sequence interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'p>: PyClass + Sized {
    fn __len__(&'p self) -> Self::Result
    where
        Self: PySequenceLenProtocol<'p>,
    {
        unimplemented!()
    }

    fn __getitem__(&'p self, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceGetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setitem__(&'p mut self, idx: Self::Index, value: Self::Value) -> Self::Result
    where
        Self: PySequenceSetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delitem__(&'p mut self, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceDelItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __contains__(&'p self, item: Self::Item) -> Self::Result
    where
        Self: PySequenceContainsProtocol<'p>,
    {
        unimplemented!()
    }

    fn __concat__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PySequenceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repeat__(&'p self, count: Self::Index) -> Self::Result
    where
        Self: PySequenceRepeatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_concat__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PySequenceInplaceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_repeat__(&'p mut self, count: Self::Index) -> Self::Result
    where
        Self: PySequenceInplaceRepeatProtocol<'p>,
    {
        unimplemented!()
    }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'p>: PySequenceProtocol<'p> {
    type Result: IntoPyCallbackOutput<usize>;
}

pub trait PySequenceGetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceSetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p> + From<isize>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PySequenceDelItemProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PySequenceContainsProtocol<'p>: PySequenceProtocol<'p> {
    type Item: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<bool>;
}

pub trait PySequenceConcatProtocol<'p>: PySequenceProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceRepeatProtocol<'p>: PySequenceProtocol<'p> {
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceInplaceConcatProtocol<'p>:
    PySequenceProtocol<'p> + IntoPy<PyObject> + 'p
{
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<Self>;
}

pub trait PySequenceInplaceRepeatProtocol<'p>:
    PySequenceProtocol<'p> + IntoPy<PyObject> + 'p
{
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<Self>;
}

#[derive(Default)]
pub struct PySequenceMethods {
    pub sq_length: Option<ffi::lenfunc>,
    pub sq_concat: Option<ffi::binaryfunc>,
    pub sq_repeat: Option<ffi::ssizeargfunc>,
    pub sq_item: Option<ffi::ssizeargfunc>,
    pub sq_ass_item: Option<ffi::ssizeobjargproc>,
    pub sq_contains: Option<ffi::objobjproc>,
    pub sq_inplace_concat: Option<ffi::binaryfunc>,
    pub sq_inplace_repeat: Option<ffi::ssizeargfunc>,
}

#[doc(hidden)]
impl PySequenceMethods {
    pub(crate) fn update_slots(&self, slots: &mut Vec<ffi::PyType_Slot>) {
        maybe_push_slot(
            slots,
            ffi::Py_sq_length,
            self.sq_length.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_concat,
            self.sq_concat.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_repeat,
            self.sq_repeat.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_item,
            self.sq_item.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_ass_item,
            self.sq_ass_item.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_contains,
            self.sq_contains.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_inplace_concat,
            self.sq_inplace_concat.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_sq_inplace_repeat,
            self.sq_inplace_repeat.map(|v| v as *mut c_void),
        );
    }

    pub fn set_len<T>(&mut self)
    where
        T: for<'p> PySequenceLenProtocol<'p>,
    {
        self.sq_length = py_len_func!(PySequenceLenProtocol, T::__len__);
    }
    pub fn set_concat<T>(&mut self)
    where
        T: for<'p> PySequenceConcatProtocol<'p>,
    {
        self.sq_concat = py_binary_func!(PySequenceConcatProtocol, T::__concat__);
    }
    pub fn set_repeat<T>(&mut self)
    where
        T: for<'p> PySequenceRepeatProtocol<'p>,
    {
        self.sq_repeat = py_ssizearg_func!(PySequenceRepeatProtocol, T::__repeat__);
    }
    pub fn set_getitem<T>(&mut self)
    where
        T: for<'p> PySequenceGetItemProtocol<'p>,
    {
        self.sq_item = py_ssizearg_func!(PySequenceGetItemProtocol, T::__getitem__);
    }
    pub fn set_setitem<T>(&mut self)
    where
        T: for<'p> PySequenceSetItemProtocol<'p>,
    {
        self.sq_ass_item = sq_ass_item_impl::set_item::<T>();
    }
    pub fn set_delitem<T>(&mut self)
    where
        T: for<'p> PySequenceDelItemProtocol<'p>,
    {
        self.sq_ass_item = sq_ass_item_impl::del_item::<T>();
    }
    pub fn set_setdelitem<T>(&mut self)
    where
        T: for<'p> PySequenceDelItemProtocol<'p> + for<'p> PySequenceSetItemProtocol<'p>,
    {
        self.sq_ass_item = sq_ass_item_impl::set_del_item::<T>();
    }
    pub fn set_contains<T>(&mut self)
    where
        T: for<'p> PySequenceContainsProtocol<'p>,
    {
        self.sq_contains = py_binary_func!(PySequenceContainsProtocol, T::__contains__, c_int);
    }
    pub fn set_inplace_concat<T>(&mut self)
    where
        T: for<'p> PySequenceInplaceConcatProtocol<'p>,
    {
        self.sq_inplace_concat = py_binary_func!(
            PySequenceInplaceConcatProtocol,
            T::__inplace_concat__,
            *mut ffi::PyObject,
            call_mut
        )
    }
    pub fn set_inplace_repeat<T>(&mut self)
    where
        T: for<'p> PySequenceInplaceRepeatProtocol<'p>,
    {
        self.sq_inplace_repeat = py_ssizearg_func!(
            PySequenceInplaceRepeatProtocol,
            T::__inplace_repeat__,
            call_mut
        )
    }
}

/// It can be possible to delete and set items (PySequenceSetItemProtocol and
/// PySequenceDelItemProtocol implemented), only to delete (PySequenceDelItemProtocol implemented)
/// or no deleting or setting is possible
mod sq_ass_item_impl {
    use super::*;

    pub(super) fn set_item<T>() -> Option<ffi::ssizeobjargproc>
    where
        T: for<'p> PySequenceSetItemProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            key: ffi::Py_ssize_t,
            value: *mut ffi::PyObject,
        ) -> c_int
        where
            T: for<'p> PySequenceSetItemProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                if value.is_null() {
                    return Err(PyErr::new::<exceptions::PyNotImplementedError, _>(format!(
                        "Item deletion is not supported by {:?}",
                        stringify!(T)
                    )));
                }

                let mut slf = slf.try_borrow_mut()?;
                let value = py.from_borrowed_ptr::<PyAny>(value);
                let value = value.extract()?;
                crate::callback::convert(py, slf.__setitem__(key.into(), value))
            })
        }
        Some(wrap::<T>)
    }

    pub(super) fn del_item<T>() -> Option<ffi::ssizeobjargproc>
    where
        T: for<'p> PySequenceDelItemProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            key: ffi::Py_ssize_t,
            value: *mut ffi::PyObject,
        ) -> c_int
        where
            T: for<'p> PySequenceDelItemProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                if value.is_null() {
                    crate::callback::convert(py, slf.borrow_mut().__delitem__(key.into()))
                } else {
                    Err(PyErr::new::<exceptions::PyNotImplementedError, _>(format!(
                        "Item assignment not supported by {:?}",
                        stringify!(T)
                    )))
                }
            })
        }
        Some(wrap::<T>)
    }

    pub(super) fn set_del_item<T>() -> Option<ffi::ssizeobjargproc>
    where
        T: for<'p> PySequenceSetItemProtocol<'p> + for<'p> PySequenceDelItemProtocol<'p>,
    {
        unsafe extern "C" fn wrap<T>(
            slf: *mut ffi::PyObject,
            key: ffi::Py_ssize_t,
            value: *mut ffi::PyObject,
        ) -> c_int
        where
            T: for<'p> PySequenceSetItemProtocol<'p> + for<'p> PySequenceDelItemProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

                if value.is_null() {
                    call_mut!(slf, __delitem__; key.into()).convert(py)
                } else {
                    let value = py.from_borrowed_ptr::<PyAny>(value);
                    let mut slf_ = slf.try_borrow_mut()?;
                    let value = value.extract()?;
                    slf_.__setitem__(key.into(), value).convert(py)
                }
            })
        }
        Some(wrap::<T>)
    }
}
