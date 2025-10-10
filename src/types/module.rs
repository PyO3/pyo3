use pyo3_ffi::c_str;

use crate::err::{PyErr, PyResult};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::callback::IntoPyCallbackOutput;
use crate::py_result_ext::PyResultExt;
use crate::pyclass::PyClass;
use crate::types::{
    any::PyAnyMethods, list::PyListMethods, PyAny, PyCFunction, PyDict, PyList, PyString,
};
use crate::{
    exceptions, ffi, Borrowed, Bound, BoundObject, IntoPyObject, IntoPyObjectExt, Py, Python,
};
#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
use std::ffi::c_int;
use std::ffi::CStr;
use std::str;

/// Represents a Python [`module`][1] object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyModule>`][crate::Py] or [`Bound<'py, PyModule>`][Bound].
///
/// For APIs available on `module` objects, see the [`PyModuleMethods`] trait which is implemented for
/// [`Bound<'py, PyModule>`][Bound].
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
    /// Creates a new module object with the `__name__` attribute set to `name`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let module = PyModule::new(py, "my_module")?;
    ///
    ///     assert_eq!(module.name()?, "my_module");
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    ///  ```
    pub fn new<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
        let name = PyString::new(py, name);
        unsafe {
            ffi::PyModule_NewObject(name.as_ptr())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }

    /// Imports the Python module with the specified name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// use pyo3::prelude::*;
    ///
    /// Python::attach(|py| {
    ///     let module = PyModule::import(py, "antigravity").expect("No flying for you.");
    /// });
    /// # }
    ///  ```
    ///
    /// This is equivalent to the following Python expression:
    /// ```python
    /// import antigravity
    /// ```
    ///
    /// If you want to import a class, you can store a reference to it with
    /// [`PyOnceLock::import`][crate::sync::PyOnceLock::import].
    pub fn import<'py, N>(py: Python<'py>, name: N) -> PyResult<Bound<'py, PyModule>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        let name = name.into_pyobject_or_pyerr(py)?;
        unsafe {
            ffi::PyImport_Import(name.as_ptr())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }

    /// Creates and loads a module named `module_name`,
    /// containing the Python code passed to `code`
    /// and pretending to live at `file_name`.
    ///
    /// If `file_name` is empty, it will be set to `<string>`.
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
    /// - Any of the arguments cannot be converted to [`CString`][std::ffi::CString]s.
    ///
    /// # Example: bundle in a file at compile time with [`include_str!`][std::include_str]:
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::ffi::c_str;
    ///
    /// # fn main() -> PyResult<()> {
    /// // This path is resolved relative to this file.
    /// let code = c_str!(include_str!("../../assets/script.py"));
    ///
    /// Python::attach(|py| -> PyResult<()> {
    ///     PyModule::from_code(py, code, c_str!("example.py"), c_str!("example"))?;
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
    /// use pyo3::ffi::c_str;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> PyResult<()> {
    /// # #[cfg(not(target_arch = "wasm32"))]  // node fs doesn't see this file, maybe cwd wrong?
    /// # {
    /// // This path is resolved by however the platform resolves paths,
    /// // which also makes this less portable. Consider using `include_str`
    /// // if you just want to bundle a script with your module.
    /// let code = std::fs::read_to_string("assets/script.py")?;
    ///
    /// Python::attach(|py| -> PyResult<()> {
    ///     PyModule::from_code(py, CString::new(code)?.as_c_str(), c_str!("example.py"), c_str!("example"))?;
    ///     Ok(())
    /// })?;
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_code<'py>(
        py: Python<'py>,
        code: &CStr,
        file_name: &CStr,
        module_name: &CStr,
    ) -> PyResult<Bound<'py, PyModule>> {
        let file_name = if file_name.is_empty() {
            c_str!("<string>")
        } else {
            file_name
        };
        unsafe {
            let code = ffi::Py_CompileString(code.as_ptr(), file_name.as_ptr(), ffi::Py_file_input)
                .assume_owned_or_err(py)?;

            ffi::PyImport_ExecCodeModuleEx(module_name.as_ptr(), code.as_ptr(), file_name.as_ptr())
                .assume_owned_or_err(py)
                .cast_into()
        }
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
    fn filename(&self) -> PyResult<Bound<'py, PyString>>;

    /// Adds an attribute to the module.
    ///
    /// For adding classes, functions or modules, prefer to use [`PyModuleMethods::add_class`],
    /// [`PyModuleMethods::add_function`] or [`PyModuleMethods::add_submodule`] instead,
    /// respectively.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
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
        N: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>;

    /// Adds a new class to the module.
    ///
    /// Notice that this method does not take an argument.
    /// Instead, this method is *generic*, and requires us to use the
    /// "turbofish" syntax to specify the class we want to add.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
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
    #[doc = concat!("[1]: https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/class.html#constructor")]
    fn add_class<T>(&self) -> PyResult<()>
    where
        T: PyClass;

    /// Adds a function or a (sub)module to a module, using the functions name as name.
    ///
    /// Prefer to use [`PyModuleMethods::add_function`] and/or [`PyModuleMethods::add_submodule`]
    /// instead.
    fn add_wrapped<T>(&self, wrapper: &impl Fn(Python<'py>) -> T) -> PyResult<()>
    where
        T: IntoPyCallbackOutput<'py, Py<PyAny>>;

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
    /// ```rust,no_run
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule]
    /// fn my_module(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     let submodule = PyModule::new(py, "submodule")?;
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
    /// ```rust,no_run
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

    /// Declare whether or not this module supports running with the GIL disabled
    ///
    /// If the module does not rely on the GIL for thread safety, you can pass
    /// `false` to this function to indicate the module does not rely on the GIL
    /// for thread-safety.
    ///
    /// This function sets the [`Py_MOD_GIL`
    /// slot](https://docs.python.org/3/c-api/module.html#c.Py_mod_gil) on the
    /// module object. The default is `Py_MOD_GIL_USED`, so passing `true` to
    /// this function is a no-op unless you have already set `Py_MOD_GIL` to
    /// `Py_MOD_GIL_NOT_USED` elsewhere.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pyo3::prelude::*;
    ///
    /// #[pymodule(gil_used = false)]
    /// fn my_module(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    ///     let submodule = PyModule::new(py, "submodule")?;
    ///     submodule.gil_used(false)?;
    ///     module.add_submodule(&submodule)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The resulting module will not print a `RuntimeWarning` and re-enable the
    /// GIL when Python imports it on the free-threaded build, since all module
    /// objects defined in the extension have `Py_MOD_GIL` set to
    /// `Py_MOD_GIL_NOT_USED`.
    ///
    /// This is a no-op on the GIL-enabled build.
    fn gil_used(&self, gil_used: bool) -> PyResult<()>;
}

impl<'py> PyModuleMethods<'py> for Bound<'py, PyModule> {
    fn dict(&self) -> Bound<'py, PyDict> {
        unsafe {
            // PyModule_GetDict returns borrowed ptr; must make owned for safety (see #890).
            ffi::PyModule_GetDict(self.as_ptr())
                .assume_borrowed(self.py())
                .to_owned()
                .cast_into_unchecked()
        }
    }

    fn index(&self) -> PyResult<Bound<'py, PyList>> {
        let __all__ = __all__(self.py());
        match self.getattr(__all__) {
            Ok(idx) => idx.cast_into().map_err(PyErr::from),
            Err(err) => {
                if err.is_instance_of::<exceptions::PyAttributeError>(self.py()) {
                    let l = PyList::empty(self.py());
                    self.setattr(__all__, &l)?;
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
                    .cast_into_unchecked()
            }
        }

        #[cfg(PyPy)]
        {
            self.dict()
                .get_item("__name__")
                .map_err(|_| exceptions::PyAttributeError::new_err("__name__"))?
                .cast_into()
                .map_err(PyErr::from)
        }
    }

    fn filename(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(PyPy))]
        unsafe {
            ffi::PyModule_GetFilenameObject(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }

        #[cfg(PyPy)]
        {
            self.dict()
                .get_item("__file__")
                .map_err(|_| exceptions::PyAttributeError::new_err("__file__"))?
                .cast_into()
                .map_err(PyErr::from)
        }
    }

    fn add<N, V>(&self, name: N, value: V) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        fn inner(
            module: &Bound<'_, PyModule>,
            name: Borrowed<'_, '_, PyString>,
            value: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<()> {
            module
                .index()?
                .append(name)
                .expect("could not append __name__ to __all__");
            module.setattr(name, value)
        }

        let py = self.py();
        inner(
            self,
            name.into_pyobject_or_pyerr(py)?.as_borrowed(),
            value.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
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
        T: IntoPyCallbackOutput<'py, Py<PyAny>>,
    {
        fn inner(module: &Bound<'_, PyModule>, object: Bound<'_, PyAny>) -> PyResult<()> {
            let name = object.getattr(__name__(module.py()))?;
            module.add(name.cast_into::<PyString>()?, object)
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
        self.add(name.cast_into::<PyString>()?, fun)
    }

    #[cfg_attr(any(Py_LIMITED_API, not(Py_GIL_DISABLED)), allow(unused_variables))]
    fn gil_used(&self, gil_used: bool) -> PyResult<()> {
        #[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
        {
            let gil_used = match gil_used {
                true => ffi::Py_MOD_GIL_USED,
                false => ffi::Py_MOD_GIL_NOT_USED,
            };
            match unsafe { ffi::PyUnstable_Module_SetGIL(self.as_ptr(), gil_used) } {
                c_int::MIN..=-1 => Err(PyErr::fetch(self.py())),
                0..=c_int::MAX => Ok(()),
            }
        }
        #[cfg(any(Py_LIMITED_API, not(Py_GIL_DISABLED)))]
        Ok(())
    }
}

fn __all__(py: Python<'_>) -> &Bound<'_, PyString> {
    intern!(py, "__all__")
}

fn __name__(py: Python<'_>) -> &Bound<'_, PyString> {
    intern!(py, "__name__")
}

#[cfg(test)]
mod tests {
    use pyo3_ffi::c_str;

    use crate::{
        types::{module::PyModuleMethods, PyModule},
        Python,
    };

    #[test]
    fn module_import_and_name() {
        Python::attach(|py| {
            let builtins = PyModule::import(py, "builtins").unwrap();
            assert_eq!(builtins.name().unwrap(), "builtins");
        })
    }

    #[test]
    fn module_filename() {
        use crate::types::string::PyStringMethods;
        Python::attach(|py| {
            let site = PyModule::import(py, "site").unwrap();
            assert!(site
                .filename()
                .unwrap()
                .to_cow()
                .unwrap()
                .ends_with("site.py"));
        })
    }

    #[test]
    fn module_from_code_empty_file() {
        Python::attach(|py| {
            let builtins = PyModule::from_code(py, c_str!(""), c_str!(""), c_str!("")).unwrap();
            assert_eq!(builtins.filename().unwrap(), "<string>");
        })
    }
}
