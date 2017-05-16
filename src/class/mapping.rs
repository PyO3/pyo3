// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject, PyDrop};
use conversion::ToPyObject;
use objects::{exc, PyObject, PyType, PyModule};
use py_class::slots::{LenResultConverter, UnitCallbackConverter};
use function::{handle_callback, PyObjectCallbackConverter};
use class::NO_METHODS;


/// Mapping interface
pub trait PyMappingProtocol {
    fn __len__(&self, py: Python) -> PyResult<usize>;

    fn __getitem__(&self, py: Python, key: &PyObject) -> PyResult<PyObject>;

    fn __setitem__(&self, py: Python, key: &PyObject, value: &PyObject) -> PyResult<()>;

    fn __delitem__(&self, py: Python, key: &PyObject) -> PyResult<()>;
}

impl<T> PyMappingProtocol for T where T: PythonObject {
    default fn __len__(&self, _py: Python) -> PyResult<usize> {
        Ok(0)
    }

    default fn __getitem__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __setitem__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(
            py, format!("Subscript assignment not supported by {:?}", self.as_object())))
    }

    default fn __delitem__(&self, py: Python, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(
            py, format!("Subscript deletion not supported by {:?}", self.as_object())))
    }
}

#[doc(hidden)]
pub trait PyMappingProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PyMappingProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
}

impl ffi::PyMappingMethods {

    /// Construct PyMappingMethods struct for PyTypeObject.tp_as_mapping
    pub fn new<T>() -> Option<ffi::PyMappingMethods>
        where T: PyMappingProtocol + PyMappingProtocolImpl + PythonObject
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut meth: ffi::PyMappingMethods = ffi::PyMappingMethods_INIT;

        for name in methods {
            match name {
                &"__len__" => {
                    meth.mp_length = py_len_func!(
                        PyMappingProtocol, T::__len__, LenResultConverter);
                },
                &"__getitem__" => {
                    meth.mp_subscript = py_binary_func!(
                        PyMappingProtocol, T::__getitem__, PyObjectCallbackConverter);
                },
                _ => unreachable!(),
            }
        }

        // always set
        meth.mp_ass_subscript = Some(mp_ass_subscript::<T>());

        Some(meth)
    }
}


fn mp_ass_subscript<T>() -> ffi::objobjargproc
    where T: PyMappingProtocol + PythonObject
{
    unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject,
                                 key: *mut ffi::PyObject,
                                 value: *mut ffi::PyObject) -> c_int
        where T: PyMappingProtocol + PythonObject
    {
        const LOCATION: &'static str = concat!(stringify!($class), ".__setitem__()");

        handle_callback(
            LOCATION, UnitCallbackConverter, |py|
            {
                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                let key = PyObject::from_borrowed_ptr(py, key);

                // if value is none, then __delitem__
                let ret = if value.is_null() {
                    slf.__delitem__(py, &key)
                } else {
                    let value = PyObject::from_borrowed_ptr(py, value);
                    let ret = slf.__setitem__(py, &key, &value);
                    PyDrop::release_ref(value, py);
                    ret
                };

                PyDrop::release_ref(key, py);
                PyDrop::release_ref(slf, py);
                ret
            })
    }
    wrap::<T>
}
