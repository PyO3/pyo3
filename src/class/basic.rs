// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! Check [the Python C API information](https://docs.python.org/3/reference/datamodel.html#basic-customization)
//! for more information.
//!
//! Parts of the documentation are copied from the respective methods from the
//! [typeobj docs](https://docs.python.org/3/c-api/typeobj.html)

use crate::{
    callback::{HashCallbackOutput, IntoPyCallbackOutput},
    derive_utils::TryFromPyCell,
};
use crate::{exceptions, ffi, FromPyObject, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;

/// Operators for the __richcmp__ method
#[derive(Debug)]
pub enum CompareOp {
    Lt = ffi::Py_LT as isize,
    Le = ffi::Py_LE as isize,
    Eq = ffi::Py_EQ as isize,
    Ne = ffi::Py_NE as isize,
    Gt = ffi::Py_GT as isize,
    Ge = ffi::Py_GE as isize,
}

/// Basic Python class customization
#[allow(unused_variables)]
pub trait PyObjectProtocol<'p>: PyClass {
    fn __getattr__(slf: Self::Receiver, name: Self::Name) -> Self::Result
    where
        Self: PyObjectGetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setattr__(slf: Self::Receiver, name: Self::Name, value: Self::Value) -> Self::Result
    where
        Self: PyObjectSetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delattr__(slf: Self::Receiver, name: Self::Name) -> Self::Result
    where
        Self: PyObjectDelAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __str__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyObjectStrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repr__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyObjectReprProtocol<'p>,
    {
        unimplemented!()
    }

    fn __format__(slf: Self::Receiver, format_spec: Self::Format) -> Self::Result
    where
        Self: PyObjectFormatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __hash__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyObjectHashProtocol<'p>,
    {
        unimplemented!()
    }

    fn __bytes__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyObjectBytesProtocol<'p>,
    {
        unimplemented!()
    }

    fn __richcmp__(slf: Self::Receiver, other: Self::Other, op: CompareOp) -> Self::Result
    where
        Self: PyObjectRichcmpProtocol<'p>,
    {
        unimplemented!()
    }
    fn __bool__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyObjectBoolProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyObjectGetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Name: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectSetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Name: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}
pub trait PyObjectDelAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Name: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}
pub trait PyObjectStrProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectReprProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectFormatProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Format: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectHashProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<HashCallbackOutput>;
}
pub trait PyObjectBoolProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<bool>;
}
pub trait PyObjectBytesProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectRichcmpProtocol<'p>: PyObjectProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

py_unary_func!(str, PyObjectStrProtocol, T::__str__);
py_unary_func!(repr, PyObjectReprProtocol, T::__repr__);
py_unary_func!(hash, PyObjectHashProtocol, T::__hash__, ffi::Py_hash_t);

#[doc(hidden)]
pub unsafe extern "C" fn getattr<T>(
    slf: *mut ffi::PyObject,
    arg: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    T: for<'p> PyObjectGetAttrProtocol<'p>,
{
    crate::callback_body!(py, {
        // Behave like python's __getattr__ (as opposed to __getattribute__) and check
        // for existing fields and methods first
        let existing = ffi::PyObject_GenericGetAttr(slf, arg);
        if existing.is_null() {
            // PyObject_HasAttr also tries to get an object and clears the error if it fails
            ffi::PyErr_Clear();
        } else {
            return Ok(existing);
        }

        let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);
        let arg = py.from_borrowed_ptr::<PyAny>(arg);
        let borrow =
            <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf).map_err(|e| e.into())?;
        T::__getattr__(borrow, arg.extract()?).convert(py)
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn richcmp<T>(
    slf: *mut ffi::PyObject,
    arg: *mut ffi::PyObject,
    op: c_int,
) -> *mut ffi::PyObject
where
    T: for<'p> PyObjectRichcmpProtocol<'p>,
{
    crate::callback_body!(py, {
        let slf = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
        let arg = extract_or_return_not_implemented!(py, arg);
        let op = match op {
            ffi::Py_LT => CompareOp::Lt,
            ffi::Py_LE => CompareOp::Le,
            ffi::Py_EQ => CompareOp::Eq,
            ffi::Py_NE => CompareOp::Ne,
            ffi::Py_GT => CompareOp::Gt,
            ffi::Py_GE => CompareOp::Ge,
            _ => {
                return Err(exceptions::PyValueError::new_err(
                    "tp_richcompare called with invalid comparison operator",
                ))
            }
        };

        let borrow =
            <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf).map_err(|e| e.into())?;

        T::__richcmp__(borrow, arg, op).convert(py)
    })
}

py_func_set!(setattr, PyObjectSetAttrProtocol, T::__setattr__);
py_func_del!(delattr, PyObjectDelAttrProtocol, T::__delattr__);
py_func_set_del!(
    setdelattr,
    PyObjectSetAttrProtocol,
    PyObjectDelAttrProtocol,
    Self,
    __setattr__,
    __delattr__
);
py_unary_func!(bool, PyObjectBoolProtocol, T::__bool__, c_int);
