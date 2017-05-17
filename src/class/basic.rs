// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use std::os::raw::c_int;

use ::{CompareOp, Py_hash_t};
use ffi;
use err::{PyErr, PyResult};
use python::{Python, PythonObject, PyDrop};
use objects::{exc, PyObject};
use conversion::ToPyObject;
use callback::{handle_callback, PyObjectCallbackConverter, HashConverter, UnitCallbackConverter};
use class::{NO_METHODS, NO_PY_METHODS};

// __new__
// __init__
// __call__
// classmethod
// staticmethod


/// Basic customization
pub trait PyObjectProtocol {

    fn __getattr__(&self, py: Python, name: &PyObject) -> PyResult<PyObject>;

    fn __setattr__(&self, py: Python, name: &PyObject, value: &PyObject) -> PyResult<()>;

    fn __delattr__(&self, py: Python, name: &PyObject) -> PyResult<()>;

    // __instancecheck__
    // __subclasscheck__
    // __iter__
    // __next__
    // __dir__

    fn __str__(&self, py: Python) -> PyResult<PyObject>;

    fn __repr__(&self, py: Python) -> PyResult<PyObject>;

    fn __hash__(&self, py: Python) -> PyResult<u64>;

    fn __bool__(&self, py: Python) -> PyResult<bool>;

    fn __richcmp__(&self, py: Python, other: &PyObject, op: CompareOp) -> PyResult<PyObject>;

}


impl<T> PyObjectProtocol for T {

    default fn __getattr__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __setattr__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __delattr__(&self, py: Python, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    // __instancecheck__
    // __subclasscheck__
    // __iter__
    // __next__
    // __dir__

    default fn __str__(&self, py: Python) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __repr__(&self, py: Python) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __hash__(&self, py: Python) -> PyResult<u64> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __bool__(&self, py: Python) -> PyResult<bool> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
    default fn __richcmp__(&self, py: Python, _: &PyObject, _: CompareOp) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
}

#[doc(hidden)]
pub trait PyObjectProtocolImpl {
    fn methods() -> &'static [&'static str];
    fn py_methods() -> &'static [::class::PyMethodDefType];
}

impl<T> PyObjectProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
    default fn py_methods() -> &'static [::class::PyMethodDefType] {
        NO_PY_METHODS
    }
}

pub fn py_object_proto_impl<T>(type_object: &mut ffi::PyTypeObject)
    where T: PyObjectProtocol + PyObjectProtocolImpl + PythonObject
{
    let methods = T::methods();
    if methods.is_empty() {
        return
    }

    for name in methods {
        match name {
            &"__str__" =>
                type_object.tp_str = py_unary_func!(
                    PyObjectProtocol, T::__str__, PyObjectCallbackConverter),
            &"__repr__" =>
                type_object.tp_repr = py_unary_func!(
                    PyObjectProtocol, T::__repr__, PyObjectCallbackConverter),
            &"__hash__" =>
                type_object.tp_hash = py_unary_func!(
                    PyObjectProtocol, T::__hash__, HashConverter, Py_hash_t),
            &"__getattr__" =>
                type_object.tp_getattro = py_binary_func!(
                    PyObjectProtocol, T::__getattr__, PyObjectCallbackConverter),
            &"__richcmp__" =>
                type_object.tp_richcompare = tp_richcompare::<T>(),
            _ => (),
        }
    }

    if methods.contains(&"__setattr__") || methods.contains(&"__getattr__") {
        type_object.tp_setattro = tp_setattro::<T>()
    }
}


fn tp_setattro<T>() -> Option<ffi::setattrofunc>
    where T: PyObjectProtocol + PythonObject
{
    unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                 key: *mut ffi::PyObject,
                                 value: *mut ffi::PyObject) -> c_int
        where T: PyObjectProtocol + PythonObject
    {
        const LOCATION: &'static str = concat!(stringify!(T), ".__setitem__()");

        handle_callback(
            LOCATION, UnitCallbackConverter, |py|
            {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let key = PyObject::from_borrowed_ptr(py, key);

                // if value is none, then __delitem__
                let ret = if value.is_null() {
                    slf.__delattr__(py, &key)
                } else {
                    let value = PyObject::from_borrowed_ptr(py, value);
                    let ret = slf.__setattr__(py, &key, &value);
                    PyDrop::release_ref(value, py);
                    ret
                };

                PyDrop::release_ref(key, py);
                PyDrop::release_ref(slf, py);
                ret
            })
    }
    Some(wrap::<T>)
}

fn extract_op(py: Python, op: c_int) -> PyResult<CompareOp> {
    match op {
        ffi::Py_LT => Ok(CompareOp::Lt),
        ffi::Py_LE => Ok(CompareOp::Le),
        ffi::Py_EQ => Ok(CompareOp::Eq),
        ffi::Py_NE => Ok(CompareOp::Ne),
        ffi::Py_GT => Ok(CompareOp::Gt),
        ffi::Py_GE => Ok(CompareOp::Ge),
        _ => Err(PyErr::new_lazy_init(
            py.get_type::<exc::ValueError>(),
            Some("tp_richcompare called with invalid comparison operator"
                 .to_py_object(py).into_object())))
    }
}

fn tp_richcompare<T>() -> Option<ffi::richcmpfunc>
    where T: PyObjectProtocol + PythonObject
{
    unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                 arg: *mut ffi::PyObject,
                                 op: c_int) -> *mut ffi::PyObject
        where T: PyObjectProtocol + PythonObject
    {
        const LOCATION: &'static str = concat!(stringify!(T), ".__richcmp__()");
        handle_callback(LOCATION, PyObjectCallbackConverter, |py| {
            let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
            let arg = PyObject::from_borrowed_ptr(py, arg);
            let ret = match extract_op(py, op) {
                Ok(op) => slf.__richcmp__(py, &arg, op),
                Err(_) => Ok(py.NotImplemented())
            };
            PyDrop::release_ref(arg, py);
            PyDrop::release_ref(slf, py);
            ret
        })
    }
    Some(wrap::<T>)
}
