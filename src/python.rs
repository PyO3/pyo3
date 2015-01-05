use std;
use std::kinds::marker::{NoSend, NoCopy, InvariantLifetime};
use std::ptr;
use ffi;
use std::c_str::CString;
use object::PyObject;

/// The 'Python' struct is a zero-size marker struct that is required for most python operations.
/// This is used to indicate that the operation accesses/modifies the python interpreter state,
/// and thus can only be called if the python interpreter is initialized and the GIL is acquired.
/// The lifetime 'p represents the lifetime of the python interpreter.
/// For example, python constants like None have the type "&'p PyObject<'p>".
/// You can imagine the GIL to be a giant "Mutex<AllPythonState>". This makes 'p the lifetime of the
/// python state protected by that mutex.
#[derive(Copy)]
pub struct Python<'p>(NoSend, InvariantLifetime<'p>);

impl<'p> Python<'p> {
    /// Retrieve python instance under the assumption that the GIL is already acquired at this point,
    /// and stays acquired for the lifetime 'p.
    pub unsafe fn assume_gil_acquired() -> Python<'p> {
        Python(NoSend, InvariantLifetime)
    }
    
    /// Retrieves a reference to the special 'None' value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_None()) }
    }
    
    /// Retrieves a reference to the 'True' constant value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_True()) }
    }
    
    /// Retrieves a reference to the 'False' constant value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_False()) }
    }
    
/*
	/// Retrieve python instance from an existing PyObject.
	/// This can be used to avoid having to explicitly pass the &Python parameter to each function call.
	/// Note that the reference may point to a different memory location than the original &Python used to
	/// construct the object -- the Python type is just used as a token to prove that the GIL is currently held.
	pub fn from_object<T : PythonObject>(_ : &T) -> &Python {
		&STATIC_PYTHON_INSTANCE
	}

	/// Acquires the global interpreter lock, which allows access to the Python runtime.
	/// This function is unsafe because 
	/// This function is unsafe because it is possible to recursively acquire the GIL,
	/// and thus getting access to multiple '&mut Python' references to the python interpreter.
	pub unsafe fn acquire_gil(&self) -> GILGuard {
		let gstate = ffi::PyGILState_Ensure(); // acquire GIL
		GILGuard { py: NEW_PYTHON_INSTANCE, gstate: gstate }
	}

	/// Releases the GIL and allows the use of python on other threads.
	pub fn allow_threads<T>(&mut self, f: fn() -> T) -> T {
		let save = unsafe { ffi::PyEval_SaveThread() };
		let result = f();
		unsafe { ffi::PyEval_RestoreThread(save); }
		result
	}

	pub fn module_type(&self) -> &PyTypeObject {
		unsafe { PyTypeObject::from_type_ptr(self, &mut ffi::PyModule_Type) }
	}

	// Importing Modules

	/// Imports the python with the given name.
	pub fn import_module<'s, N : ToCStr>(&'s self, name : N) -> PyResult<'s, PyPtr<'s, PyModule>> {
	use module;
		name.with_c_str(|name| unsafe {
			let m = ffi::PyImport_ImportModule(name);
			let m : PyPtr<PyObject> = try!(err::result_from_owned_ptr(self, m));
			module::as_module(self, m)
		})
	}

	/// Create a new module object based on a name.
	pub fn init_module<Sized? N : ToCStr>
		(&self, name : &N, doc : Option<&CString>) -> PyResult<&PyModule>
	{
		let name = name.to_c_str();
		unsafe {
			ffi::PyEval_InitThreads();
			let m = ffi::Py_InitModule3(name.as_ptr(), ptr::null_mut(), doc.as_ptr());
			if m.is_null() {
				Err(PyErr::fetch(self))
			} else {
				Ok(PythonObject::from_ptr(self, m))
			}
		}
	}
	*/
}

/*
/// RAII type that represents an acquired GIL.
#[must_use]
pub struct GILGuard {
	gstate : ffi::PyGILState_STATE,
	py : Python
}

#[unsafe_destructor]
impl Drop for GILGuard {
	fn drop(&mut self) {
		unsafe { ffi::PyGILState_Release(self.gstate) }
	}
}

impl GILGuard {
	pub fn python(&mut self) -> &mut Python {
		&mut self.py
	}
}*/

