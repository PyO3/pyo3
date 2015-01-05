use ffi;
use std::kinds::marker::{NoSend, NoCopy};
use python::Python;

/// Represents the python interpreter instance.
/// The python runtime is initialized using `PythonInterpreter::new()`,
/// and destroyed when the PythonInterpreter is dropped.
pub struct PythonInterpreter(NoSend, NoCopy);

#[must_use]
impl PythonInterpreter {
	/// Initializes the python interpreter.
	/// Unsafe because we currently do not prevent multiple initialization, which is not supported.
	pub unsafe fn new() -> PythonInterpreter {
		ffi::Py_Initialize();
		ffi::PyEval_InitThreads();
		PythonInterpreter(NoSend, NoCopy)
	}

	pub fn python<'p>(&'p self) -> Python<'p> {
		unsafe { Python::assume_gil_acquired() }
	}
}

#[unsafe_destructor]
impl Drop for PythonInterpreter {
	fn drop(&mut self) {
		unsafe { ffi::Py_Finalize() }
	}
}


