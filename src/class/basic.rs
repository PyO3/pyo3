// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! Check [the Python C API information](https://docs.python.org/3/reference/datamodel.html#basic-customization)
//! for more information.
//!
//! Parts of the documentation are copied from the respective methods from the
//! [typeobj docs](https://docs.python.org/3/c-api/typeobj.html)

use crate::callback::{HashCallbackOutput, IntoPyCallbackOutput};
use crate::{exceptions, ffi, FromPyObject, PyAny, PyCell, PyClass, PyObject, PyResult};
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

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyBasicSlots {
    fn get_str() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectStrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_str,
            pfunc: py_unary_func!(PyObjectStrProtocol, Self::__str__) as _,
        }
    }

    fn get_repr() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectReprProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_repr,
            pfunc: py_unary_func!(PyObjectReprProtocol, Self::__repr__) as _,
        }
    }

    fn get_hash() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectHashProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_hash,
            pfunc: py_unary_func!(PyObjectHashProtocol, Self::__hash__, ffi::Py_hash_t) as _,
        }
    }

    fn get_getattr() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectGetAttrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_getattro,
            pfunc: tp_getattro::<Self>() as _,
        }
    }

    fn get_richcompare() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectRichcmpProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_getattro,
            pfunc: tp_richcompare::<Self>() as _,
        }
    }

    fn get_setattr() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectSetAttrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_setattro,
            pfunc: py_func_set!(PyObjectSetAttrProtocol, Self::__setattr__) as _,
        }
    }

    fn get_delattr() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectDelAttrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_setattro,
            pfunc: py_func_del!(PyObjectDelAttrProtocol, Self::__delattr__) as _,
        }
    }

    fn get_setdelattr() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectSetAttrProtocol<'p> + for<'p> PyObjectDelAttrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_setattro,
            pfunc: py_func_set_del!(
                PyObjectSetAttrProtocol,
                PyObjectDelAttrProtocol,
                Self,
                __setattr__,
                __delattr__
            ) as _,
        }
    }

    fn get_bool() -> ffi::PyType_Slot
    where
        Self: for<'p> PyObjectBoolProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_bool,
            pfunc: py_unary_func!(PyObjectBoolProtocol, Self::__bool__, c_int) as _,
        }
    }
}

impl<'p, T> PyBasicSlots for T where T: PyObjectProtocol<'p> {}

fn tp_getattro<T>() -> ffi::binaryfunc
where
    T: for<'p> PyObjectGetAttrProtocol<'p>,
{
    unsafe extern "C" fn wrap<T>(
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
    wrap::<T>
}

fn tp_richcompare<T>() -> ffi::richcmpfunc
where
    T: for<'p> PyObjectRichcmpProtocol<'p>,
{
    fn extract_op(op: c_int) -> PyResult<CompareOp> {
        match op {
            ffi::Py_LT => Ok(CompareOp::Lt),
            ffi::Py_LE => Ok(CompareOp::Le),
            ffi::Py_EQ => Ok(CompareOp::Eq),
            ffi::Py_NE => Ok(CompareOp::Ne),
            ffi::Py_GT => Ok(CompareOp::Gt),
            ffi::Py_GE => Ok(CompareOp::Ge),
            _ => Err(exceptions::PyValueError::new_err(
                "tp_richcompare called with invalid comparison operator",
            )),
        }
    }
    unsafe extern "C" fn wrap<T>(
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
            let op = extract_op(op)?;

            slf.try_borrow()?.__richcmp__(arg, op).convert(py)
        })
    }
    wrap::<T>
}
