use std;
use ffi;
//use ffi::CStringAsPtr;
//use {PyResult, PyErr, FromPyObject, PyTypeObject, ToPyObject, PyObject, Python, PyPtr, PythonObject};
use {Python, PyPtr, PyResult, PyObject, PythonObject, PyTypeObject};
use err;

pub struct PyModule<'p>(PyObject<'p>);

impl <'p> PythonObject<'p> for PyModule<'p> {
    fn from_object<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyModule<'p>> {
        unsafe {
            if ffi::PyModule_Check(obj.as_ptr()) {
                Some(std::mem::transmute(obj))
            } else {
                None
            }
        }
    }
    
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        &self.0
    }
    
    fn type_object(_ : Option<&Self>) -> &'p PyTypeObject<'p> {
        panic!()
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

