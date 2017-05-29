// Copyright (c) 2017-present PyO3 Project and Contributors

use std::fmt;
use std::marker::PhantomData;

use ffi;
use pyptr::{Py, PyPtr};
use err::{PyResult};
use python::{Python, ToPythonPointer, PyDowncastInto};
use objects::PyString;
use typeob::{PyTypeInfo, PyObjectAlloc};


pub struct PythonToken<T>(PhantomData<T>);

impl<T> PythonToken<T> {
    pub fn token<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

#[inline]
pub fn with_token<'p, T, F>(py: Python<'p>, f: F) -> Py<'p, T>
    where F: FnOnce(PythonToken<T>) -> T,
          T: PyTypeInfo + PyObjectAlloc<Type=T>
{
    let value = f(PythonToken(PhantomData));
    if let Ok(ob) = Py::new(py, value) {
        ob
    } else {
        ::err::panic_after_error()
    }
}


pub trait PythonObjectWithGilToken<'p> : Sized {
    fn gil(&self) -> Python<'p>;
}

pub trait PythonObjectWithToken : Sized {
    fn token<'p>(&'p self) -> Python<'p>;
}

pub struct PyObjectMarker;


impl PyObjectMarker {

    #[inline]
    pub fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_owned_ptr(ptr) }
    }

    #[inline]
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject)
                                 -> PyResult<PyPtr<PyObjectMarker>> {
        unsafe { PyPtr::from_owned_ptr_or_err(py, ptr) }
    }

    #[inline]
    pub fn from_owned_ptr_or_opt(py: Python, ptr: *mut ffi::PyObject)
                                 -> Option<PyPtr<PyObjectMarker>> {
        unsafe { PyPtr::from_owned_ptr_or_opt(py, ptr) }
    }

    #[inline]
    pub fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_borrowed_ptr(ptr) }
    }
}


impl<'p> fmt::Debug for PyPtr<PyObjectMarker> {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let repr_obj = unsafe {
            PyString::downcast_from_owned_ptr(py, ffi::PyObject_Repr(self.as_ptr()))
                .map_err(|_| fmt::Error)?
        };
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl<'p> fmt::Display for PyPtr<PyObjectMarker> {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let str_obj = unsafe {
            PyString::downcast_from_owned_ptr(py, ffi::PyObject_Str(self.as_ptr()))
                .map_err(|_| fmt::Error)?
        };
        f.write_str(&str_obj.to_string_lossy())
    }
}
