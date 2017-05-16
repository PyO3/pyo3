// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Sequence Interface
//! Trait and support implementation for implementing sequence

use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject, PyDrop};
use conversion::ToPyObject;
use objects::{exc, PyObject, PyType, PyModule};
use py_class::slots::{LenResultConverter, UnitCallbackConverter, BoolConverter};
use function::{handle_callback, PyObjectCallbackConverter};
use class::NO_METHODS;


/// Mapping interface
pub trait PySequenceProtocol {
    fn __len__(&self, py: Python) -> PyResult<usize>;

    fn __getitem__(&self, py: Python, key: isize) -> PyResult<PyObject>;

    fn __setitem__(&self, py: Python, key: isize, value: &PyObject) -> PyResult<()>;

    fn __delitem__(&self, py: Python, key: isize) -> PyResult<()>;

    fn __contains__(&self, py: Python, value: &PyObject) -> PyResult<bool>;

    fn __concat__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;

    fn __repeat__(&self, py: Python, count: isize) -> PyResult<PyObject>;

    fn __inplace_concat__(&self, py: Python, other: &PyObject) -> PyResult<PyObject>;

    fn __inplace_repeat__(&self, py: Python, count: isize) -> PyResult<PyObject>;

}

impl<T> PySequenceProtocol for T where T: PythonObject {
    default fn __len__(&self, py: Python) -> PyResult<usize> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __getitem__(&self, py: Python, _: isize) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __setitem__(&self, py: Python, _: isize, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(
            py, format!("Subscript assignment not supported by {:?}", self.as_object())))
    }

    default fn __delitem__(&self, py: Python, _: isize) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(
            py, format!("Subscript deletion not supported by {:?}", self.as_object())))
    }

    default fn __contains__(&self, py: Python, _: &PyObject) -> PyResult<bool> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __concat__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __repeat__(&self, py: Python, _: isize) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __inplace_concat__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __inplace_repeat__(&self, py: Python, _: isize) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
}

#[doc(hidden)]
pub trait PySequenceProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PySequenceProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
}

impl ffi::PySequenceMethods {

    /// Construct PySequenceMethods struct for PyTypeObject.tp_as_sequence
    pub fn new<T>() -> Option<ffi::PySequenceMethods>
        where T: PySequenceProtocol + PySequenceProtocolImpl + PythonObject
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut meth: ffi::PySequenceMethods = ffi::PySequenceMethods_INIT;

        for name in methods {
            match name {
                &"__len__" => {
                    meth.sq_length = py_len_func!(
                        PySequenceProtocol, T::__len__, LenResultConverter);
                },
                &"__getitem__" => {
                    meth.sq_item = py_ssizearg_func!(
                        PySequenceProtocol, T::__getitem__, PyObjectCallbackConverter);
                },
                &"__repeat__" => {
                    meth.sq_repeat = py_ssizearg_func!(
                        PySequenceProtocol, T::__repeat__, PyObjectCallbackConverter);
                },
                &"__contains__" => {
                    meth.sq_contains = py_objobj_proc!(
                        PySequenceProtocol, T::__contains__, BoolConverter);
                },
                &"__concat__" => {
                    meth.sq_concat = py_binary_func!(
                        PySequenceProtocol, T::__concat__, PyObjectCallbackConverter);
                },
                &"__inplace_concat__" => {
                    meth.sq_inplace_concat = py_binary_func!(
                        PySequenceProtocol, T::__inplace_concat__, PyObjectCallbackConverter);
                },
                &"__inplace_repeat__" => {
                    meth.sq_inplace_repeat = py_ssizearg_func!(
                        PySequenceProtocol, T::__inplace_repeat__, PyObjectCallbackConverter);
                },
                _ => unreachable!(),
            }
        }

        // always set
        meth.sq_ass_item = Some(sq_ass_subscript::<T>());

        Some(meth)
    }
}


fn sq_ass_subscript<T>() -> ffi::ssizeobjargproc
    where T: PySequenceProtocol + PythonObject
{
    unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                 key: ffi::Py_ssize_t,
                                 value: *mut ffi::PyObject) -> c_int
        where T: PySequenceProtocol + PythonObject
    {
        const LOCATION: &'static str = concat!(stringify!($class), ".__setitem__()");

        handle_callback(
            LOCATION, UnitCallbackConverter, |py|
            {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();

                // if value is none, then __delitem__
                let ret = if value.is_null() {
                    slf.__delitem__(py, key as isize)
                } else {
                    let value = PyObject::from_borrowed_ptr(py, value);
                    let ret = slf.__setitem__(py, key as isize, &value);
                    PyDrop::release_ref(value, py);
                    ret
                };

                PyDrop::release_ref(slf, py);
                ret
            })
    }
    wrap::<T>
}
