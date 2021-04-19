// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use crate::conversion::{FromPyObject, IntoPy};
use crate::err::PyErr;
use crate::{callback::IntoPyCallbackOutput, derive_utils::TryFromPyCell};
use crate::{exceptions, ffi, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;

/// Sequence interface
#[allow(unused_variables)]
pub trait PySequenceProtocol<'p>: PyClass + Sized {
    fn __len__(slf: Self::Receiver) -> Self::Result
    where
        Self: PySequenceLenProtocol<'p>,
    {
        unimplemented!()
    }

    fn __getitem__(slf: Self::Receiver, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceGetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setitem__(slf: Self::Receiver, idx: Self::Index, value: Self::Value) -> Self::Result
    where
        Self: PySequenceSetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delitem__(slf: Self::Receiver, idx: Self::Index) -> Self::Result
    where
        Self: PySequenceDelItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __contains__(slf: Self::Receiver, item: Self::Item) -> Self::Result
    where
        Self: PySequenceContainsProtocol<'p>,
    {
        unimplemented!()
    }

    fn __concat__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PySequenceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repeat__(slf: Self::Receiver, count: Self::Index) -> Self::Result
    where
        Self: PySequenceRepeatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_concat__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PySequenceInplaceConcatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __inplace_repeat__(slf: Self::Receiver, count: Self::Index) -> Self::Result
    where
        Self: PySequenceInplaceRepeatProtocol<'p>,
    {
        unimplemented!()
    }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PySequenceLenProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<usize>;
}

pub trait PySequenceGetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceSetItemProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Index: FromPyObject<'p> + From<isize>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PySequenceDelItemProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PySequenceContainsProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Item: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<bool>;
}

pub trait PySequenceConcatProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceRepeatProtocol<'p>: PySequenceProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Index: FromPyObject<'p> + From<isize>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PySequenceInplaceConcatProtocol<'p>:
    PySequenceProtocol<'p> + IntoPy<PyObject> + 'p
{
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<Self>;
}

pub trait PySequenceInplaceRepeatProtocol<'p>:
    PySequenceProtocol<'p> + IntoPy<PyObject> + 'p
{
    type Receiver: TryFromPyCell<'p, Self>;
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

        let borrow =
            <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf).map_err(|e| e.into())?;
        let value = py.from_borrowed_ptr::<PyAny>(value);
        let value = value.extract()?;
        crate::callback::convert(py, T::__setitem__(borrow, key.into(), value))
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
            let borrow =
                <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf).map_err(|e| e.into())?;
            crate::callback::convert(py, T::__delitem__(borrow, key.into()))
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
            let borrow =
                <<T as PySequenceDelItemProtocol>::Receiver as TryFromPyCell<_>>::try_from_pycell(
                    slf,
                )
                .map_err(|e| e.into())?;
            T::__delitem__(borrow, key.into()).convert(py)
        } else {
            let value = py.from_borrowed_ptr::<PyAny>(value);
            let value = value.extract()?;
            let borrow =
                <<T as PySequenceSetItemProtocol>::Receiver as TryFromPyCell<_>>::try_from_pycell(
                    slf,
                )
                .map_err(|e| e.into())?;
            T::__setitem__(borrow, key.into(), value).convert(py)
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
    *mut ffi::PyObject
);
py_ssizearg_func!(
    inplace_repeat,
    PySequenceInplaceRepeatProtocol,
    Self::__inplace_repeat__
);
