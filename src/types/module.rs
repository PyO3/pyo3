// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::callback::IntoPyCallbackOutput;
use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi;
use crate::instance::PyNativeType;
use crate::pyclass::PyClass;
use crate::type_object::PyTypeObject;
use crate::types::{PyAny, PyDict, PyList};
use crate::types::{PyCFunction, PyTuple};
use crate::{AsPyPointer, IntoPy, Py, PyObject, Python};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str;

/// Represents a Python `module` object.
#[repr(transparent)]
pub struct PyModule(PyAny);

pyobject_native_var_type!(PyModule, ffi::PyModule_Type, ffi::PyModule_Check);

impl PyModule {
    /// Creates a new module object with the `__name__` attribute set to name.
    pub fn new<'p>(py: Python<'p>, name: &str) -> PyResult<&'p PyModule> {
        // Could use PyModule_NewObject, but it doesn't exist on PyPy.
        let name = CString::new(name)?;
        unsafe { py.from_owned_ptr_or_err(ffi::PyModule_New(name.as_ptr())) }
    }

    /// Imports the Python module with the specified name.
    pub fn import<'p>(py: Python<'p>, name: &str) -> PyResult<&'p PyModule> {
        crate::types::with_tmp_string(py, name, |name| unsafe {
            py.from_owned_ptr_or_err(ffi::PyImport_Import(name))
        })
    }

    /// Loads the Python code specified into a new module.
    ///
    /// `code` is the raw Python you want to load into the module.
    /// `file_name` is the file name to associate with the module
    /// (this is used when Python reports errors, for example).
    /// `module_name` is the name to give the module.
    pub fn from_code<'p>(
        py: Python<'p>,
        code: &str,
        file_name: &str,
        module_name: &str,
    ) -> PyResult<&'p PyModule> {
        let data = CString::new(code)?;
        let filename = CString::new(file_name)?;
        let module = CString::new(module_name)?;

        unsafe {
            let cptr = ffi::Py_CompileString(data.as_ptr(), filename.as_ptr(), ffi::Py_file_input);
            if cptr.is_null() {
                return Err(PyErr::fetch(py));
            }

            let mptr = ffi::PyImport_ExecCodeModuleEx(module.as_ptr(), cptr, filename.as_ptr());
            if mptr.is_null() {
                return Err(PyErr::fetch(py));
            }

            <&PyModule as crate::FromPyObject>::extract(py.from_owned_ptr_or_err(mptr)?)
        }
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self) -> &PyDict {
        unsafe {
            // PyModule_GetDict returns borrowed ptr; must make owned for safety (see #890).
            let ptr = ffi::PyModule_GetDict(self.as_ptr());
            ffi::Py_INCREF(ptr);
            self.py().from_owned_ptr(ptr)
        }
    }

    /// Return the index (`__all__`) of the module, creating one if needed.
    pub fn index(&self) -> PyResult<&PyList> {
        match self.getattr("__all__") {
            Ok(idx) => idx.downcast().map_err(PyErr::from),
            Err(err) => {
                if err.is_instance::<exceptions::PyAttributeError>(self.py()) {
                    let l = PyList::empty(self.py());
                    self.setattr("__all__", l).map_err(PyErr::from)?;
                    Ok(l)
                } else {
                    Err(err)
                }
            }
        }
    }

    unsafe fn str_from_ptr(&self, ptr: *const c_char) -> PyResult<&str> {
        if ptr.is_null() {
            Err(PyErr::fetch(self.py()))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::from_instance(
                    exceptions::PyUnicodeDecodeError::new_utf8(self.py(), slice, e)?,
                )),
            }
        }
    }

    /// Returns the module's name.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name(&self) -> PyResult<&str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetName(self.as_ptr())) }
    }

    /// Returns the module's filename.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    #[cfg(not(all(windows, PyPy)))]
    pub fn filename(&self) -> PyResult<&str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Calls a function in the module.
    ///
    /// This is equivalent to the Python expression `module.name(*args, **kwargs)`.
    pub fn call(
        &self,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        self.getattr(name)?.call(args, kwargs)
    }

    /// Calls a function in the module with only positional arguments.
    ///
    /// This is equivalent to the Python expression `module.name(*args)`.
    pub fn call1(&self, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
        self.getattr(name)?.call1(args)
    }

    /// Calls a function in the module without arguments.
    ///
    /// This is equivalent to the Python expression `module.name()`.
    pub fn call0(&self, name: &str) -> PyResult<&PyAny> {
        self.getattr(name)?.call0()
    }

    /// Gets a member from the module.
    ///
    /// This is equivalent to the Python expression `module.name`.
    pub fn get(&self, name: &str) -> PyResult<&PyAny> {
        self.getattr(name)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<()>
    where
        V: IntoPy<PyObject>,
    {
        self.index()?
            .append(name)
            .expect("could not append __name__ to __all__");
        self.setattr(name, value.into_py(self.py()))
    }

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that initializes the `class`,
    /// sets `new_type.__module__` to this module's name,
    /// and adds the type to this module.
    pub fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyClass,
    {
        self.add(T::NAME, <T as PyTypeObject>::type_object(self.py()))
    }

    /// Adds a function or a (sub)module to a module, using the functions __name__ as name.
    ///
    /// Use this together with the`#[pyfunction]` and [wrap_pyfunction!] or `#[pymodule]` and
    /// [wrap_pymodule!].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// #[pymodule]
    /// fn utils(_py: Python, _module: &PyModule) -> PyResult<()> {
    ///     Ok(())
    /// }
    ///
    /// #[pyfunction]
    /// fn double(x: usize) -> usize {
    ///     x * 2
    /// }
    /// #[pymodule]
    /// fn top_level(_py: Python, module: &PyModule) -> PyResult<()> {
    ///     module.add_wrapped(pyo3::wrap_pymodule!(utils))?;
    ///     module.add_wrapped(pyo3::wrap_pyfunction!(double))
    /// }
    /// ```
    ///
    /// You can also add a function with a custom name using [add](PyModule::add):
    ///
    /// ```rust,ignore
    /// m.add("also_double", wrap_pyfunction!(double)(m)?)?;
    /// ```
    ///
    /// **This function will be deprecated in the next release. Please use the specific
    /// [PyModule::add_function] and [PyModule::add_submodule] functions instead.**
    pub fn add_wrapped<'a, T>(&'a self, wrapper: &impl Fn(Python<'a>) -> T) -> PyResult<()>
    where
        T: IntoPyCallbackOutput<PyObject>,
    {
        let py = self.py();
        let function = wrapper(py).convert(py)?;
        let name = function.getattr(py, "__name__")?;
        let name = name.extract(py)?;
        self.add(name, function)
    }

    /// Add a submodule to a module.
    ///
    /// Use this together with `#[pymodule]` and [wrap_pymodule!].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// fn init_utils(module: &PyModule) -> PyResult<()> {
    ///     module.add("super_useful_constant", "important")
    /// }
    /// #[pymodule]
    /// fn top_level(py: Python, module: &PyModule) -> PyResult<()> {
    ///     let utils = PyModule::new(py, "utils")?;
    ///     init_utils(utils)?;
    ///     module.add_submodule(utils)
    /// }
    /// ```
    pub fn add_submodule(&self, module: &PyModule) -> PyResult<()> {
        let name = module.name()?;
        self.add(name, module)
    }

    /// Add a function to a module.
    ///
    /// Use this together with the`#[pyfunction]` and [wrap_pyfunction!].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// #[pyfunction]
    /// fn double(x: usize) -> usize {
    ///     x * 2
    /// }
    /// #[pymodule]
    /// fn double_mod(_py: Python, module: &PyModule) -> PyResult<()> {
    ///     module.add_function(pyo3::wrap_pyfunction!(double, module)?)
    /// }
    /// ```
    ///
    /// You can also add a function with a custom name using [add](PyModule::add):
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// #[pyfunction]
    /// fn double(x: usize) -> usize {
    ///     x * 2
    /// }
    /// #[pymodule]
    /// fn double_mod(_py: Python, module: &PyModule) -> PyResult<()> {
    ///     module.add("also_double", pyo3::wrap_pyfunction!(double, module)?)
    /// }
    /// ```
    pub fn add_function<'a>(&'a self, fun: &'a PyCFunction) -> PyResult<()> {
        let name = fun.getattr("__name__")?.extract()?;
        self.add(name, fun)
    }
}
