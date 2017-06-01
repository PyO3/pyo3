// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::fmt;

use ffi;
use typeob::PyTypeInfo;
use pointers::{Py, Ptr, PyPtr};
use python::{Python, PyDowncastInto, ToPythonPointer};
use objectprotocol::ObjectProtocol;


impl<'p> std::fmt::Debug for Ptr<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let repr = unsafe { ::PyString::downcast_from_owned_ptr(
            self.token(), ffi::PyObject_Repr(self.as_ptr())) };
        let repr = repr.map_err(|_| std::fmt::Error)?;
        f.write_str(&repr.to_string_lossy())
    }
}

impl<'p> std::fmt::Display for Ptr<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let ob = unsafe { ::PyString::downcast_from_owned_ptr(
            self.token(), ffi::PyObject_Str(self.as_ptr())) };
        let ob = ob.map_err(|_| std::fmt::Error)?;
        f.write_str(&ob.to_string_lossy())
    }
}

impl<'p, T> fmt::Debug for Py<'p, T> where T: ObjectProtocol<'p> + PyTypeInfo {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when repr() fails
        let repr_obj = try!(self.repr().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl<'p, T> fmt::Display for Py<'p, T> where T: ObjectProtocol<'p> + PyTypeInfo {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when str() fails
        let str_obj = try!(self.str().map_err(|_| fmt::Error));
        f.write_str(&str_obj.to_string_lossy())
    }
}

impl fmt::Debug for PyPtr {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_object(py);
        let repr_obj = try!(r.repr().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl fmt::Display for PyPtr {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_object(py);
        let repr_obj = try!(r.str().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}
