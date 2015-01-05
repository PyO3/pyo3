use std;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject};
use object::PyObject;
use typeobject::PyType;
use pyptr::PyPtr;
use err::{self, PyResult};

pub struct PyModule<'p>(PyObject<'p>);

impl <'p> PythonObject<'p> for PyModule<'p> {
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        &self.0
    }
    
    #[inline]
    unsafe fn unchecked_downcast_from<'a>(obj: &'a PyObject<'p>) -> &'a PyModule<'p> {
        std::mem::transmute(obj)
    }
}

impl <'p> PythonObjectWithCheckedDowncast<'p> for PyModule<'p> {
    #[inline]
    fn downcast_from<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyModule<'p>> {
        unsafe {
            if ffi::PyModule_Check(obj.as_ptr()) {
                Some(PythonObject::unchecked_downcast_from(obj))
            } else {
                None
            }
        }
    }
}

impl <'p> PythonObjectWithTypeObject<'p> for PyModule<'p> {
    #[inline]
    fn type_object(py: Python<'p>, _ : Option<&Self>) -> &'p PyType<'p> {
        unsafe { PyType::from_type_ptr(py, &mut ffi::PyModule_Type) }
    }
}

impl <'p> PyModule<'p> {
    pub fn import<N : std::c_str::ToCStr>(py : Python<'p>, name : N) -> PyResult<PyPtr<PyModule<'p>>> {
        let result = name.with_c_str(|name| unsafe {
            err::result_from_owned_ptr(py, ffi::PyImport_ImportModule(name))
        });
        try!(result).downcast_into()
    }
}

/*
pub fn as_module<'p>(py : &'p Python, obj : PyPtr<PyObject>) -> PyResult<'p, PyPtr<'p, PyModule>> {
	if py.module_type().is_instance(obj.deref()) {
		Ok(unsafe { PyPtr::from_owned_ptr(py, obj.steal_ptr()) })
	} else {
		Err(PyErr::type_error(py, obj.deref(), py.module_type()))
	}
}

impl PyModule {

	pub fn add_object<Sized? S : ToCStr, Sized? T : ToPyObject>
		(&self, name : &S, value : &T) -> PyResult<()>
	{
		let value = try!(value.to_py_object(self.python())).steal_ptr();
		let rc = name.with_c_str(|name| unsafe {
			ffi::PyModule_AddObject(self.as_ptr(), name, value)
		});
		err::result_from_error_code(self.python(), rc)
	}

}
*/

