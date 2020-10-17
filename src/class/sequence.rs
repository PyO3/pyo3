// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use crate::callback::IntoPyCallbackOutput;
use crate::conversion::{FromPyObject, IntoPy};
use crate::err::PyErr;
use crate::{exceptions, ffi, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;
#[cfg(Py_LIMITED_API)]
use std::os::raw::c_void;

#[cfg(Py_LIMITED_API)]
#[derive(Clone)]
pub struct PySequenceMethods {
    pub sq_length: Option<ffi::lenfunc>,
    pub sq_concat: Option<ffi::binaryfunc>,
    pub sq_repeat: Option<ffi::ssizeargfunc>,
    pub sq_item: Option<ffi::ssizeargfunc>,
    #[allow(dead_code)]
    pub was_sq_slice: *mut c_void,
    pub sq_ass_item: Option<ffi::ssizeobjargproc>,
    #[allow(dead_code)]
    pub was_sq_ass_slice: *mut c_void,
    pub sq_contains: Option<ffi::objobjproc>,
    pub sq_inplace_concat: Option<ffi::binaryfunc>,
    pub sq_inplace_repeat: Option<ffi::ssizeargfunc>,
}

#[cfg(not(Py_LIMITED_API))]
pub use ffi::PySequenceMethods;

impl Default for PySequenceMethods {
    fn default() -> Self {
        Self {
            sq_length: None,
            sq_concat: None,
            sq_repeat: None,
            sq_item: None,
            was_sq_slice: std::ptr::null_mut(),
            sq_ass_item: None,
            was_sq_ass_slice: std::ptr::null_mut(),
            sq_contains: None,
            sq_inplace_concat: None,
            sq_inplace_repeat: None,
        }
    }
}

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

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PySequenceSlots {
    fn get_len() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceLenProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_len_func!(PySequenceLenProtocol, Self::__len__) as _,
        }
    }
    fn get_concat() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceConcatProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_binary_func!(PySequenceConcatProtocol, Self::__concat__) as _,
        }
    }
    fn get_repeat() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceRepeatProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_ssizearg_func!(PySequenceRepeatProtocol, Self::__repeat__) as _,
        }
    }
    fn get_getitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceGetItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_ssizearg_func!(PySequenceGetItemProtocol, Self::__getitem__) as _,
        }
    }
    fn get_setitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceSetItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: sq_ass_item_impl::set_item::<Self>() as _,
        }
    }
    fn get_delitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceDelItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: sq_ass_item_impl::del_item::<Self>() as _,
        }
    }
    fn get_setdelitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceDelItemProtocol<'p> + for<'p> PySequenceSetItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: sq_ass_item_impl::set_del_item::<Self>() as _,
        }
    }
    fn get_contains() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceContainsProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_binary_func!(PySequenceContainsProtocol, Self::__contains__, c_int) as _,
        }
    }
    fn get_inplace_concat() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceInplaceConcatProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_binary_func!(
                PySequenceInplaceConcatProtocol,
                Self::__inplace_concat__,
                *mut ffi::PyObject,
                call_mut
            ) as _,
        }
    }
    fn get_inplace_repeat() -> ffi::PyType_Slot
    where
        Self: for<'p> PySequenceInplaceRepeatProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_sq_length,
            pfunc: py_ssizearg_func!(
                PySequenceInplaceRepeatProtocol,
                Self::__inplace_repeat__,
                call_mut
            ) as _,
        }
    }
}

impl<'p, T> PySequenceSlots for T where T: PySequenceProtocol<'p> {}

/// It can be possible to delete and set items (PySequenceSetItemProtocol and
/// PySequenceDelItemProtocol implemented), only to delete (PySequenceDelItemProtocol implemented)
/// or no deleting or setting is possible
mod sq_ass_item_impl {
    use super::*;

    pub(super) fn set_item<T>() -> ffi::ssizeobjargproc
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
                    return Err(exceptions::PyNotImplementedError::new_err(format!(
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
        wrap::<T>
    }

    pub(super) fn del_item<T>() -> ffi::ssizeobjargproc
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
        wrap::<T>
    }

    pub(super) fn set_del_item<T>() -> ffi::ssizeobjargproc
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
        wrap::<T>
    }
}
