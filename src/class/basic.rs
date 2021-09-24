// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! Check [the Python C API information](https://docs.python.org/3/reference/datamodel.html#basic-customization)
//! for more information.
//!
//! Parts of the documentation are copied from the respective methods from the
//! [typeobj docs](https://docs.python.org/3/c-api/typeobj.html)

use crate::callback::{HashCallbackOutput, IntoPyCallbackOutput};
use crate::{exceptions, ffi, FromPyObject, PyAny, PyCell, PyClass, PyObject};
use std::os::raw::c_int;

/// Operators for the `__richcmp__` method
#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    /// The *less than* operator.
    Lt = ffi::Py_LT as isize,
    /// The *less than or equal to* operator.
    Le = ffi::Py_LE as isize,
    /// The equality operator.
    Eq = ffi::Py_EQ as isize,
    /// The *not equal to* operator.
    Ne = ffi::Py_NE as isize,
    /// The *greater than* operator.
    Gt = ffi::Py_GT as isize,
    /// The *greater than or equal to* operator.
    Ge = ffi::Py_GE as isize,
}

impl CompareOp {
    pub fn from_raw(op: c_int) -> Option<Self> {
        match op {
            ffi::Py_LT => Some(CompareOp::Lt),
            ffi::Py_LE => Some(CompareOp::Le),
            ffi::Py_EQ => Some(CompareOp::Eq),
            ffi::Py_NE => Some(CompareOp::Ne),
            ffi::Py_GT => Some(CompareOp::Gt),
            ffi::Py_GE => Some(CompareOp::Ge),
            _ => None,
        }
    }
}

/// Basic Python class customization
#[allow(unused_variables)]
pub trait PyObjectProtocol<'p>: PyClass {
    fn __getattr__(&'p self, name: Self::Name) -> Self::Result
    where
        Self: PyObjectGetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setattr__(&'p mut self, name: Self::Name, value: Self::Value) -> Self::Result
    where
        Self: PyObjectSetAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delattr__(&'p mut self, name: Self::Name) -> Self::Result
    where
        Self: PyObjectDelAttrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __str__(&'p self) -> Self::Result
    where
        Self: PyObjectStrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __repr__(&'p self) -> Self::Result
    where
        Self: PyObjectReprProtocol<'p>,
    {
        unimplemented!()
    }

    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__format__` in `#[pymethods]` instead of in a protocol"
    )]
    fn __format__(&'p self, format_spec: Self::Format) -> Self::Result
    where
        Self: PyObjectFormatProtocol<'p>,
    {
        unimplemented!()
    }

    fn __hash__(&'p self) -> Self::Result
    where
        Self: PyObjectHashProtocol<'p>,
    {
        unimplemented!()
    }

    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__bytes__` in `#[pymethods]` instead of in a protocol"
    )]
    fn __bytes__(&'p self) -> Self::Result
    where
        Self: PyObjectBytesProtocol<'p>,
    {
        unimplemented!()
    }

    fn __richcmp__(&'p self, other: Self::Other, op: CompareOp) -> Self::Result
    where
        Self: PyObjectRichcmpProtocol<'p>,
    {
        unimplemented!()
    }
    fn __bool__(&'p self) -> Self::Result
    where
        Self: PyObjectBoolProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyObjectGetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectSetAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}
pub trait PyObjectDelAttrProtocol<'p>: PyObjectProtocol<'p> {
    type Name: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}
pub trait PyObjectStrProtocol<'p>: PyObjectProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectReprProtocol<'p>: PyObjectProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectFormatProtocol<'p>: PyObjectProtocol<'p> {
    type Format: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectHashProtocol<'p>: PyObjectProtocol<'p> {
    type Result: IntoPyCallbackOutput<HashCallbackOutput>;
}
pub trait PyObjectBoolProtocol<'p>: PyObjectProtocol<'p> {
    type Result: IntoPyCallbackOutput<bool>;
}
pub trait PyObjectBytesProtocol<'p>: PyObjectProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}
pub trait PyObjectRichcmpProtocol<'p>: PyObjectProtocol<'p> {
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
        call_ref!(slf, __getattr__, arg).convert(py)
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

        slf.try_borrow()?.__richcmp__(arg, op).convert(py)
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
