use std;
use ffi;
use python::{Python, PythonObject, PythonObjectDowncast};
use object::PyObject;
use typeobject::PyType;
use pyptr::PyPtr;
use err::{self, PyResult};

pub struct PyDict<'p>(PyObject<'p>);

impl <'p> PythonObject<'p> for PyDict<'p> {
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        &self.0
    }
    
    #[inline]
    unsafe fn unchecked_downcast_from<'a>(obj: &'a PyObject<'p>) -> &'a PyDict<'p> {
        std::mem::transmute(obj)
    }
}

impl <'p> PythonObjectDowncast<'p> for PyDict<'p> {
    #[inline]
    fn downcast_from<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyDict<'p>> {
        unsafe {
            if ffi::PyDict_Check(obj.as_ptr()) {
                Some(std::mem::transmute(obj))
            } else {
                None
            }
        }
    }
    
    #[inline]
    fn type_object(py: Python<'p>, _ : Option<&Self>) -> &'p PyType<'p> {
        unsafe { PyType::from_type_ptr(py, &mut ffi::PyDict_Type) }
    }
}

impl <'p> PyDict<'p> {
    fn new(py: Python<'p>) -> PyResult<'p, PyPtr<'p, PyDict<'p>>> {
        unimplemented!()
    }
}

