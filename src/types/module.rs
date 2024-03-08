use crate::callback::IntoPyCallbackOutput;
use crate::err::{PyErr, PyResult};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::pyclass::PyClass;
use crate::types::{
    any::PyAnyMethods, list::PyListMethods, PyAny, PyCFunction, PyDict, PyList, PyString,
};
use crate::{exceptions, ffi, Bound, IntoPy, Py, PyNativeType, PyObject, Python};
use std::ffi::CString;
use std::str;

use super::PyStringMethods;

/// Represents a Python [`module`][1] object.
///
/// As with all other Python objects, modules are first class citizens.
/// This means they can be passed to or returned from functions,
/// created dynamically, assigned to variables and so forth.
///
/// [1]: https://docs.python.org/3/tutorial/modules.html
#[repr(transparent)]
pub struct PyModule(PyAny);

pyobject_native_type_core!(PyModule, pyobject_native_static_type_object!(ffi::PyModule_Type), #checkfunction=ffi::PyModule_Check);

impl PyModule {
    /// Deprecated form of [`PyModule::new_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyModule::new` will be replaced by `PyModule::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<'py>(py: Python<'py>, name: &str) -> PyResult<&'py PyModule> {
        Self::new_bound(py, name).map(Bound::into_gil_ref)
    }

    /// Creates a new module object with the `__name__` attribute set to `name`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let module = PyModule::new_bound(py, "my_module")?;
    ///
    ///     assert_eq!(module.name()?.to_cow()?, "my_module");
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    ///  ```
    pub fn new_bound<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
        // Could use PyModule_NewObject, but it doesn't exist on PyPy.
        let name = CString::new(name)?;
        unsafe {
            ffi::PyModule_New(name.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated form of [`PyModule::import_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyModule::import` will be replaced by `PyModule::import_bound` in a future PyO3 version"
        )
    )]
    pub fn import<N>(py: Python<'_>, name: N) -> PyResult<&PyModule>
    where
        N: IntoPy<Py<PyString>>,
    {
        Self::import_bound(py, name).map(Bound::into_gil_ref)
    }

    /// Imports the Python module with the specified name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// use pyo3::prelude::*;
    ///
    /// Python::with_gil(|py| {
    ///     let module = PyModule::import_bound(py, "antigravity").expect("No flying for you.");
    /// });
    /// # }
    ///  ```
    ///
    /// This is equivalent to the following Python expression:
    /// ```python
    /// import antigravity
    /// ```
    pub fn import_bound<N>(py: Python<'_>, name: N) -> PyResult<Bound<'_, PyModule>>
    where
        N: IntoPy<Py<PyString>>,
    {
        let name: Py<PyString> = name.into_py(py);
        unsafe {
            ffi::PyImport_Import(name.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated form of [`PyModule::from_code_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyModule::from_code` will be replaced by `PyModule::from_code_bound` in a future PyO3 version"
        )
    )]
    pub fn from_code<'py>(
        py: Python<'py>,
        code: &str,
        file_name: &str,
        module_name: &str,
    ) -> PyResult<&'py PyModule> {
        Self::from_code_bound(py, code, file_name, module_name).map(Bound::into_gil_ref)
    }

    /// Creates and loads a module named `module_name`,
    /// containing the Python code passed to `code`
    /// and pretending to live at `file_name`.
    ///
    /// <div class="information">
    ///     <div class="tooltip compile_fail" style="">&#x26a0; &#xfe0f;</div>
    /// </div><div class="example-wrap" style="display:inline-block"><pre class="compile_fail" style="white-space:normal;font:inherit;">
    //
    ///  <strong>Warning</strong>: This will compile and execute code. <strong>Never</strong> pass untrusted code to this function!
    ///
    /// </pre></div>
    ///
    /// # Errors
    ///
    /// Returns `PyErr` if:
    /// - `code` is not syntactically correct Python.
    /// - Any Python exceptions are raised while initializing the module.
    /// - Any of the arguments cannot be converted to [`CString`]s.
    ///
    /// # Example: bundle in a file at compile time with [`include_str!`][std::include_str]:
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// // This path is resolved relative to this file.
    /// let code = include_str!("../../assets/script.py");
    ///
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     PyModule::from_code_bound(py, code, "example.py", "example")?;
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example: Load a file at runtime with [`std::fs::read_to_string`].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// // This path is resolved by however the platform resolves paths,
    /// // which also makes this less portable. Consider using `include_str`
    /// // if you just want to bundle a script with your module.
    /// let code = std::fs::read_to_string("assets/script.py")?;
    ///
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     PyModule::from_code_bound(py, &code, "example.py", "example")?;
    ///     Ok(())
    /// })?;
    /// Ok(())
    /// # }
    /// ```
    pub fn from_code_bound<'py>(
        py: Python<'py>,
        code: &str,
        file_name: &str,
        module_name: &str,
    ) -> PyResult<Bound<'py, PyModule>> {
        let data = CString::new(code)?;
        let filename = CString::new(file_name)?;
        let module = CString::new(module_name)?;

        unsafe {
            let code = ffi::Py_CompileString(data.as_ptr(), filename.as_ptr(), ffi::Py_file_input)
                .assume_owned_or_err(py)?;

            ffi::PyImport_ExecCodeModuleEx(module.as_ptr(), code.as_ptr(), filename.as_ptr())
                .assume_owned_or_err(py)
                .downcast_into()
        }
    }

    /// Returns the module's `__dict__` attribute, which contains the module's symbol table.
    pub fn dict(&self) -> &PyDict {
        self.as_borrowed().dict().into_gil_ref()
    }

    /// Returns the index (the `__all__` attribute) of the module,
    /// creating one if needed.
    ///
    /// `__all__` declares the items that will be imported with `from my_module import *`.
    pub fn index(&self) -> PyResult<&PyList> {
        self.as_borrowed().index().map(Bound::into_gil_ref)
    }

    /// Returns the name (the `__name__` attribute) of the module.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name(&self) -> PyResult<&str> {
        self.as_borrowed().name()?.into_gil_ref().to_str()
    }

    /// Returns the filename (the `__file__` attribute) of the module.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    #[cfg(not(PyPy))]
    pub fn filename(&self) -> PyResult<&str> {
        self.as_borrowed().filename()?.into_gil_ref().to_str()
    }

    /// Adds an attribute to the module.
    ///
    /// For adding classes, functions or modules, prefer to use [`PyModule::add_class`],
    /// [`PyModule::add_function`] or [`PyModule::add_submodule`] instead, respectively.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add("c", 299_792_458)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// from my_module import c
    ///
    /// print("c is", c)
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// c is 299792458
    /// ```
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<()>
    where
        V: IntoPy<PyObject>,
    {
        self.as_borrowed().add(name, value)
    }

    /// Adds a new class to the module.
    ///
    /// Notice that this method does not take an argument.
    /// Instead, this method is *generic*, and requires us to use the
    /// "turbofish" syntax to specify the class we want to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add_class::<Foo>()?;
    ///     Ok(())
    /// }
    ///  ```
    ///
    /// Python code can see this class as such:
    /// ```python
    /// from my_module import Foo
    ///
    /// print("Foo is", Foo)
    /// ```
    ///
    /// This will result in the following output:
    /// ```text
    /// Foo is <class 'builtins.Foo'>
    /// ```
    ///
    /// Note that as we haven't defined a [constructor][1], Python code can't actually
    /// make an *instance* of `Foo` (or *get* one for that matter, as we haven't exported
    /// anything that can return instances of `Foo`).
    ///
    /// [1]: https://pyo3.rs/latest/class.html#constructor
    pub fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyClass,
    {
        self.as_borrowed().add_class::<T>()
    }

    /// Adds a function or a (sub)module to a module, using the functions name as name.
    ///
    /// Prefer to use [`PyModule::add_function`] and/or [`PyModule::add_submodule`] instead.
    pub fn add_wrapped<'a, T>(&'a self, wrapper: &impl Fn(Python<'a>) -> T) -> PyResult<()>
    where
        T: IntoPyCallbackOutput<PyObject>,
    {
        self.as_borrowed().add_wrapped(wrapper)
    }

    /// Adds a submodule to a module.
    ///
    /// This is especially useful for creating module hierarchies.
    ///
    /// Note that this doesn't define a *package*, so this won't allow Python code
    /// to directly import submodules by using
    /// <span style="white-space: pre">`from my_module import submodule`</span>.
    /// For more information, see [#759][1] and [#1517][2].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule]
    /// fn my_module(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     let submodule = PyModule::new_bound(py, "submodule")?;
    ///     submodule.add("super_useful_constant", "important")?;
    ///
    ///     module.add_submodule(&submodule)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// import my_module
    ///
    /// print("super_useful_constant is", my_module.submodule.super_useful_constant)
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// super_useful_constant is important
    /// ```
    ///
    /// [1]: https://github.com/PyO3/pyo3/issues/759
    /// [2]: https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021
    pub fn add_submodule(&self, module: &PyModule) -> PyResult<()> {
        self.as_borrowed().add_submodule(&module.as_borrowed())
    }

    /// Add a function to a module.
    ///
    /// Note that this also requires the [`wrap_pyfunction!`][2] macro
    /// to wrap a function annotated with [`#[pyfunction]`][1].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyfunction]
    /// fn say_hello() {
    ///     println!("Hello world!")
    /// }
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add_function(wrap_pyfunction!(say_hello, module)?)
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// from my_module import say_hello
    ///
    /// say_hello()
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// Hello world!
    /// ```
    ///
    /// [1]: crate::prelude::pyfunction
    /// [2]: crate::wrap_pyfunction
    pub fn add_function<'a>(&'a self, fun: &'a PyCFunction) -> PyResult<()> {
        let name = fun
            .as_borrowed()
            .getattr(__name__(self.py()))?
            .downcast_into::<PyString>()?;
        let name = name.to_cow()?;
        self.add(&name, fun)
    }
}

/// Implementation of functionality for [`PyModule`].
///
/// These methods are defined for the `Bound<'py, PyModule>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyModule")]
pub trait PyModuleMethods<'py>: crate::sealed::Sealed {
    /// Returns the module's `__dict__` attribute, which contains the module's symbol table.
    fn dict(&self) -> Bound<'py, PyDict>;

    /// Returns the index (the `__all__` attribute) of the module,
    /// creating one if needed.
    ///
    /// `__all__` declares the items that will be imported with `from my_module import *`.
    fn index(&self) -> PyResult<Bound<'py, PyList>>;

    /// Returns the name (the `__name__` attribute) of the module.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    fn name(&self) -> PyResult<Bound<'py, PyString>>;

    /// Returns the filename (the `__file__` attribute) of the module.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    #[cfg(not(PyPy))]
    fn filename(&self) -> PyResult<Bound<'py, PyString>>;

    /// Adds an attribute to the module.
    ///
    /// For adding classes, functions or modules, prefer to use [`PyModule::add_class`],
    /// [`PyModule::add_function`] or [`PyModule::add_submodule`] instead, respectively.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add("c", 299_792_458)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// from my_module import c
    ///
    /// print("c is", c)
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// c is 299792458
    /// ```
    fn add<N, V>(&self, name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: IntoPy<PyObject>;

    /// Adds a new class to the module.
    ///
    /// Notice that this method does not take an argument.
    /// Instead, this method is *generic*, and requires us to use the
    /// "turbofish" syntax to specify the class we want to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add_class::<Foo>()?;
    ///     Ok(())
    /// }
    ///  ```
    ///
    /// Python code can see this class as such:
    /// ```python
    /// from my_module import Foo
    ///
    /// print("Foo is", Foo)
    /// ```
    ///
    /// This will result in the following output:
    /// ```text
    /// Foo is <class 'builtins.Foo'>
    /// ```
    ///
    /// Note that as we haven't defined a [constructor][1], Python code can't actually
    /// make an *instance* of `Foo` (or *get* one for that matter, as we haven't exported
    /// anything that can return instances of `Foo`).
    ///
    /// [1]: https://pyo3.rs/latest/class.html#constructor
    fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyClass;

    /// Adds a function or a (sub)module to a module, using the functions name as name.
    ///
    /// Prefer to use [`PyModule::add_function`] and/or [`PyModule::add_submodule`] instead.
    fn add_wrapped<T>(&self, wrapper: &impl Fn(Python<'py>) -> T) -> PyResult<()>
    where
        T: IntoPyCallbackOutput<PyObject>;

    /// Adds a submodule to a module.
    ///
    /// This is especially useful for creating module hierarchies.
    ///
    /// Note that this doesn't define a *package*, so this won't allow Python code
    /// to directly import submodules by using
    /// <span style="white-space: pre">`from my_module import submodule`</span>.
    /// For more information, see [#759][1] and [#1517][2].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule]
    /// fn my_module(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     let submodule = PyModule::new_bound(py, "submodule")?;
    ///     submodule.add("super_useful_constant", "important")?;
    ///
    ///     module.add_submodule(&submodule)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// import my_module
    ///
    /// print("super_useful_constant is", my_module.submodule.super_useful_constant)
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// super_useful_constant is important
    /// ```
    ///
    /// [1]: https://github.com/PyO3/pyo3/issues/759
    /// [2]: https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021
    fn add_submodule(&self, module: &Bound<'_, PyModule>) -> PyResult<()>;

    /// Add a function to a module.
    ///
    /// Note that this also requires the [`wrap_pyfunction!`][2] macro
    /// to wrap a function annotated with [`#[pyfunction]`][1].
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyfunction]
    /// fn say_hello() {
    ///     println!("Hello world!")
    /// }
    /// #[pymodule]
    /// fn my_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     module.add_function(wrap_pyfunction!(say_hello, module)?)
    /// }
    /// ```
    ///
    /// Python code can then do the following:
    ///
    /// ```python
    /// from my_module import say_hello
    ///
    /// say_hello()
    /// ```
    ///
    /// This will result in the following output:
    ///
    /// ```text
    /// Hello world!
    /// ```
    ///
    /// [1]: crate::prelude::pyfunction
    /// [2]: crate::wrap_pyfunction
    fn add_function(&self, fun: Bound<'_, PyCFunction>) -> PyResult<()>;
}

impl<'py> PyModuleMethods<'py> for Bound<'py, PyModule> {
    fn dict(&self) -> Bound<'py, PyDict> {
        unsafe {
            // PyModule_GetDict returns borrowed ptr; must make owned for safety (see #890).
            ffi::PyModule_GetDict(self.as_ptr())
                .assume_borrowed(self.py())
                .to_owned()
                .downcast_into_unchecked()
        }
    }

    fn index(&self) -> PyResult<Bound<'py, PyList>> {
        let __all__ = __all__(self.py());
        match self.getattr(__all__) {
            Ok(idx) => idx.downcast_into().map_err(PyErr::from),
            Err(err) => {
                if err.is_instance_of::<exceptions::PyAttributeError>(self.py()) {
                    let l = PyList::empty_bound(self.py());
                    self.setattr(__all__, &l).map_err(PyErr::from)?;
                    Ok(l)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn name(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(PyPy))]
        {
            unsafe {
                ffi::PyModule_GetNameObject(self.as_ptr())
                    .assume_owned_or_err(self.py())
                    .downcast_into_unchecked()
            }
        }

        #[cfg(PyPy)]
        {
            self.dict()
                .get_item("__name__")
                .map_err(|_| exceptions::PyAttributeError::new_err("__name__"))?
                .downcast_into()
                .map_err(PyErr::from)
        }
    }

    #[cfg(not(PyPy))]
    fn filename(&self) -> PyResult<Bound<'py, PyString>> {
        unsafe {
            ffi::PyModule_GetFilenameObject(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn add<N, V>(&self, name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: IntoPy<PyObject>,
    {
        fn inner(
            module: &Bound<'_, PyModule>,
            name: Bound<'_, PyString>,
            value: Bound<'_, PyAny>,
        ) -> PyResult<()> {
            module
                .index()?
                .append(&name)
                .expect("could not append __name__ to __all__");
            module.setattr(name, value.into_py(module.py()))
        }

        let py = self.py();
        inner(
            self,
            name.into_py(py).into_bound(py),
            value.into_py(py).into_bound(py),
        )
    }

    fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyClass,
    {
        let py = self.py();
        self.add(T::NAME, T::lazy_type_object().get_or_try_init(py)?)
    }

    fn add_wrapped<T>(&self, wrapper: &impl Fn(Python<'py>) -> T) -> PyResult<()>
    where
        T: IntoPyCallbackOutput<PyObject>,
    {
        fn inner(module: &Bound<'_, PyModule>, object: Bound<'_, PyAny>) -> PyResult<()> {
            let name = object.getattr(__name__(module.py()))?;
            module.add(name.downcast_into::<PyString>()?, object)
        }

        let py = self.py();
        inner(self, wrapper(py).convert(py)?.into_bound(py))
    }

    fn add_submodule(&self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        let name = module.name()?;
        self.add(name, module)
    }

    fn add_function(&self, fun: Bound<'_, PyCFunction>) -> PyResult<()> {
        let name = fun.getattr(__name__(self.py()))?;
        self.add(name.downcast_into::<PyString>()?, fun)
    }
}

fn __all__(py: Python<'_>) -> &Bound<'_, PyString> {
    intern!(py, "__all__")
}

fn __name__(py: Python<'_>) -> &Bound<'_, PyString> {
    intern!(py, "__name__")
}

#[cfg(test)]
#[cfg_attr(not(feature = "gil-refs"), allow(deprecated))]
mod tests {
    use crate::{
        types::{module::PyModuleMethods, string::PyStringMethods, PyModule},
        Python,
    };

    #[test]
    fn module_import_and_name() {
        Python::with_gil(|py| {
            let builtins = PyModule::import_bound(py, "builtins").unwrap();
            assert_eq!(
                builtins.name().unwrap().to_cow().unwrap().as_ref(),
                "builtins"
            );
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn module_filename() {
        Python::with_gil(|py| {
            let site = PyModule::import_bound(py, "site").unwrap();
            assert!(site
                .filename()
                .unwrap()
                .to_cow()
                .unwrap()
                .ends_with("site.py"));
        })
    }
}
