// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use crate::callback::IntoPyCallbackOutput;
use crate::conversion::{FromPyObject, IntoPy};
use crate::err::PyErr;
use crate::{exceptions, ffi, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;

/// Sequence interface
#[allow(unused_variables)]
#[deprecated(since = "0.16.0", note = "prefer `#[pymethods]` to `#[pyproto]`")]
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

py_len_func!(len, PySequenceLenProtocol, Self::__len__);
py_binary_func!(concat, PySequenceConcatProtocol, Self::__concat__);
py_ssizearg_func!(repeat, PySequenceRepeatProtocol, Self::__repeat__);
py_ssizearg_func!(getitem, PySequenceGetItemProtocol, Self::__getitem__);

#[doc(hidden)]
pub unsafe extern "C" fn setitem<T>(
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

#[doc(hidden)]
pub unsafe extern "C" fn delitem<T>(
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

#[doc(hidden)]
pub unsafe extern "C" fn setdelitem<T>(
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

py_binary_func!(
    contains,
    PySequenceContainsProtocol,
    Self::__contains__,
    c_int
);
py_binary_func!(
    inplace_concat,
    PySequenceInplaceConcatProtocol,
    Self::__inplace_concat__,
    *mut ffi::PyObject,
    call_mut
);
py_ssizearg_func!(
    inplace_repeat,
    PySequenceInplaceRepeatProtocol,
    Self::__inplace_repeat__,
    call_mut
);
