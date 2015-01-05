use std;
use ffi;
use python::Python;
use objects::{PyObject, PyType};
use pyptr::PyPtr;
use err::{self, PyResult};

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);


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

