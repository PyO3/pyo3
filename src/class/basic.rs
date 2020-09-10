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

/// All FFI functions for basic protocols.
#[derive(Default)]
pub struct PyObjectMethods {
    pub tp_str: Option<ffi::reprfunc>,
    pub tp_repr: Option<ffi::reprfunc>,
    pub tp_hash: Option<ffi::hashfunc>,
    pub tp_getattro: Option<ffi::getattrofunc>,
    pub tp_richcompare: Option<ffi::richcmpfunc>,
    pub tp_setattro: Option<ffi::setattrofunc>,
    pub nb_bool: Option<ffi::inquiry>,
}

#[doc(hidden)]
impl PyObjectMethods {
    pub(crate) fn update_typeobj(&self, type_object: &mut ffi::PyTypeObject) {
        type_object.tp_str = self.tp_str;
        type_object.tp_repr = self.tp_repr;
        type_object.tp_hash = self.tp_hash;
        type_object.tp_getattro = self.tp_getattro;
        type_object.tp_richcompare = self.tp_richcompare;
        type_object.tp_setattro = self.tp_setattro;
    }
    // Set functions used by `#[pyproto]`.
    pub fn set_str<T>(&mut self)
    where
        T: for<'p> PyObjectStrProtocol<'p>,
    {
        self.tp_str = py_unary_func!(PyObjectStrProtocol, T::__str__);
    }
    pub fn set_repr<T>(&mut self)
    where
        T: for<'p> PyObjectReprProtocol<'p>,
    {
        self.tp_repr = py_unary_func!(PyObjectReprProtocol, T::__repr__);
    }
    pub fn set_hash<T>(&mut self)
    where
        T: for<'p> PyObjectHashProtocol<'p>,
    {
        self.tp_hash = py_unary_func!(PyObjectHashProtocol, T::__hash__, ffi::Py_hash_t);
    }
    pub fn set_getattr<T>(&mut self)
    where
        T: for<'p> PyObjectGetAttrProtocol<'p>,
    {
        self.tp_getattro = tp_getattro::<T>();
    }
    pub fn set_richcompare<T>(&mut self)
    where
        T: for<'p> PyObjectRichcmpProtocol<'p>,
    {
        self.tp_richcompare = tp_richcompare::<T>();
    }
    pub fn set_setattr<T>(&mut self)
    where
        T: for<'p> PyObjectSetAttrProtocol<'p>,
    {
        self.tp_setattro = py_func_set!(PyObjectSetAttrProtocol, T, __setattr__);
    }
    pub fn set_delattr<T>(&mut self)
    where
        T: for<'p> PyObjectDelAttrProtocol<'p>,
    {
        self.tp_setattro = py_func_del!(PyObjectDelAttrProtocol, T, __delattr__);
    }
    pub fn set_setdelattr<T>(&mut self)
    where
        T: for<'p> PyObjectSetAttrProtocol<'p> + for<'p> PyObjectDelAttrProtocol<'p>,
    {
        self.tp_setattro = py_func_set_del!(
            PyObjectSetAttrProtocol,
            PyObjectDelAttrProtocol,
            T,
            __setattr__,
            __delattr__
        )
    }
    pub fn set_bool<T>(&mut self)
    where
        T: for<'p> PyObjectBoolProtocol<'p>,
    {
        self.nb_bool = py_unary_func!(PyObjectBoolProtocol, T::__bool__, c_int);
    }
}

fn tp_getattro<T>() -> Option<ffi::binaryfunc>
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
    Some(wrap::<T>)
}

fn tp_richcompare<T>() -> Option<ffi::richcmpfunc>
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
    Some(wrap::<T>)
}
