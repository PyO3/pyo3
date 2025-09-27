use crate::err::{self, PyResult};
use crate::instance::Borrowed;
#[cfg(not(Py_3_13))]
use crate::pybacked::PyBackedStr;
#[cfg(any(Py_LIMITED_API, PyPy, not(Py_3_13)))]
use crate::types::any::PyAnyMethods;
use crate::types::PyTuple;
use crate::{ffi, Bound, PyAny, PyTypeInfo, Python};

use super::PyString;

/// Represents a reference to a Python `type` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyType>`][crate::Py] or [`Bound<'py, PyType>`][Bound].
///
/// For APIs available on `type` objects, see the [`PyTypeMethods`] trait which is implemented for
/// [`Bound<'py, PyType>`][Bound].
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, pyobject_native_static_type_object!(ffi::PyType_Type), #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeInfo>(py: Python<'_>) -> Bound<'_, PyType> {
        T::type_object(py)
    }

    /// Converts the given FFI pointer into `Bound<PyType>`, to use in safe code.
    ///
    /// The function creates a new reference from the given pointer, and returns
    /// it as a `Bound<PyType>`.
    ///
    /// # Safety
    /// - The pointer must be a valid non-null reference to a `PyTypeObject`
    #[inline]
    pub unsafe fn from_borrowed_type_ptr(
        py: Python<'_>,
        p: *mut ffi::PyTypeObject,
    ) -> Bound<'_, PyType> {
        unsafe {
            Borrowed::from_ptr_unchecked(py, p.cast())
                .cast_unchecked()
                .to_owned()
        }
    }
}

/// Implementation of functionality for [`PyType`].
///
/// These methods are defined for the `Bound<'py, PyType>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyType")]
pub trait PyTypeMethods<'py>: crate::sealed::Sealed {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    fn as_type_ptr(&self) -> *mut ffi::PyTypeObject;

    /// Gets the name of the `PyType`. Equivalent to `self.__name__` in Python.
    fn name(&self) -> PyResult<Bound<'py, PyString>>;

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    /// Equivalent to `self.__qualname__` in Python.
    fn qualname(&self) -> PyResult<Bound<'py, PyString>>;

    /// Gets the name of the module defining the `PyType`.
    fn module(&self) -> PyResult<Bound<'py, PyString>>;

    /// Gets the [fully qualified name](https://peps.python.org/pep-0737/#add-pytype-getfullyqualifiedname-function) of the `PyType`.
    fn fully_qualified_name(&self) -> PyResult<Bound<'py, PyString>>;

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    fn is_subclass(&self, other: &Bound<'_, PyAny>) -> PyResult<bool>;

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo;

    /// Return the method resolution order for this type.
    ///
    /// Equivalent to the Python expression `self.__mro__`.
    fn mro(&self) -> Bound<'py, PyTuple>;

    /// Return Python bases
    ///
    /// Equivalent to the Python expression `self.__bases__`.
    fn bases(&self) -> Bound<'py, PyTuple>;
}

impl<'py> PyTypeMethods<'py> for Bound<'py, PyType> {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Gets the name of the `PyType`.
    fn name(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(Py_3_11))]
        let name = self.getattr(intern!(self.py(), "__name__"))?.cast_into()?;

        #[cfg(Py_3_11)]
        let name = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            ffi::PyType_GetName(self.as_type_ptr())
                .assume_owned_or_err(self.py())?
                // SAFETY: setting `__name__` from Python is required to be a `str`
                .cast_into_unchecked()
        };

        Ok(name)
    }

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    fn qualname(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(Py_3_11))]
        let name = self
            .getattr(intern!(self.py(), "__qualname__"))?
            .cast_into()?;

        #[cfg(Py_3_11)]
        let name = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            ffi::PyType_GetQualName(self.as_type_ptr())
                .assume_owned_or_err(self.py())?
                // SAFETY: setting `__qualname__` from Python is required to be a `str`
                .cast_into_unchecked()
        };

        Ok(name)
    }

    /// Gets the name of the module defining the `PyType`.
    fn module(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(Py_3_13))]
        let name = self.getattr(intern!(self.py(), "__module__"))?;

        #[cfg(Py_3_13)]
        let name = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            ffi::PyType_GetModuleName(self.as_type_ptr()).assume_owned_or_err(self.py())?
        };

        // `__module__` is never guaranteed to be a `str`
        name.cast_into().map_err(Into::into)
    }

    /// Gets the [fully qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    fn fully_qualified_name(&self) -> PyResult<Bound<'py, PyString>> {
        #[cfg(not(Py_3_13))]
        let name = {
            let module = self.getattr(intern!(self.py(), "__module__"))?;
            let qualname = self.getattr(intern!(self.py(), "__qualname__"))?;

            let module_str = module.extract::<PyBackedStr>()?;
            if module_str == "builtins" || module_str == "__main__" {
                qualname.cast_into()?
            } else {
                PyString::new(self.py(), &format!("{module}.{qualname}"))
            }
        };

        #[cfg(Py_3_13)]
        let name = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            ffi::PyType_GetFullyQualifiedName(self.as_type_ptr())
                .assume_owned_or_err(self.py())?
                .cast_into_unchecked()
        };

        Ok(name)
    }

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    fn is_subclass(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsSubclass(self.as_ptr(), other.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo,
    {
        self.is_subclass(&T::type_object(self.py()))
    }

    fn mro(&self) -> Bound<'py, PyTuple> {
        #[cfg(any(Py_LIMITED_API, PyPy))]
        let mro = self
            .getattr(intern!(self.py(), "__mro__"))
            .expect("Cannot get `__mro__` from object.")
            .extract()
            .expect("Unexpected type in `__mro__` attribute.");

        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        let mro = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            (*self.as_type_ptr())
                .tp_mro
                .assume_borrowed(self.py())
                .to_owned()
                .cast_into_unchecked()
        };

        mro
    }

    fn bases(&self) -> Bound<'py, PyTuple> {
        #[cfg(any(Py_LIMITED_API, PyPy))]
        let bases = self
            .getattr(intern!(self.py(), "__bases__"))
            .expect("Cannot get `__bases__` from object.")
            .extract()
            .expect("Unexpected type in `__bases__` attribute.");

        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        let bases = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            (*self.as_type_ptr())
                .tp_bases
                .assume_borrowed(self.py())
                .to_owned()
                .cast_into_unchecked()
        };

        bases
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::generate_unique_module_name;
    use crate::types::{PyAnyMethods, PyBool, PyInt, PyModule, PyTuple, PyType, PyTypeMethods};
    use crate::PyAny;
    use crate::Python;
    use pyo3_ffi::c_str;

    #[test]
    fn test_type_is_subclass() {
        Python::attach(|py| {
            let bool_type = py.get_type::<PyBool>();
            let long_type = py.get_type::<PyInt>();
            assert!(bool_type.is_subclass(&long_type).unwrap());
        });
    }

    #[test]
    fn test_type_is_subclass_of() {
        Python::attach(|py| {
            assert!(py.get_type::<PyBool>().is_subclass_of::<PyInt>().unwrap());
        });
    }

    #[test]
    fn test_mro() {
        Python::attach(|py| {
            assert!(py
                .get_type::<PyBool>()
                .mro()
                .eq(PyTuple::new(
                    py,
                    [
                        py.get_type::<PyBool>(),
                        py.get_type::<PyInt>(),
                        py.get_type::<PyAny>()
                    ]
                )
                .unwrap())
                .unwrap());
        });
    }

    #[test]
    fn test_bases_bool() {
        Python::attach(|py| {
            assert!(py
                .get_type::<PyBool>()
                .bases()
                .eq(PyTuple::new(py, [py.get_type::<PyInt>()]).unwrap())
                .unwrap());
        });
    }

    #[test]
    fn test_bases_object() {
        Python::attach(|py| {
            assert!(py
                .get_type::<PyAny>()
                .bases()
                .eq(PyTuple::empty(py))
                .unwrap());
        });
    }

    #[test]
    fn test_type_names_standard() {
        Python::attach(|py| {
            let module_name = generate_unique_module_name("test_module");
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class MyClass:
    pass
"#
                ),
                c_str!(file!()),
                &module_name,
            )
            .expect("module create failed");

            let my_class = module.getattr("MyClass").unwrap();
            let my_class_type = my_class.cast_into::<PyType>().unwrap();
            assert_eq!(my_class_type.name().unwrap(), "MyClass");
            assert_eq!(my_class_type.qualname().unwrap(), "MyClass");
            let module_name = module_name.to_str().unwrap();
            let qualname = format!("{module_name}.MyClass");
            assert_eq!(my_class_type.module().unwrap(), module_name);
            assert_eq!(
                my_class_type.fully_qualified_name().unwrap(),
                qualname.as_str()
            );
        });
    }

    #[test]
    fn test_type_names_builtin() {
        Python::attach(|py| {
            let bool_type = py.get_type::<PyBool>();
            assert_eq!(bool_type.name().unwrap(), "bool");
            assert_eq!(bool_type.qualname().unwrap(), "bool");
            assert_eq!(bool_type.module().unwrap(), "builtins");
            assert_eq!(bool_type.fully_qualified_name().unwrap(), "bool");
        });
    }

    #[test]
    fn test_type_names_nested() {
        Python::attach(|py| {
            let module_name = generate_unique_module_name("test_module");
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class OuterClass:
    class InnerClass:
        pass
"#
                ),
                c_str!(file!()),
                &module_name,
            )
            .expect("module create failed");

            let outer_class = module.getattr("OuterClass").unwrap();
            let inner_class = outer_class.getattr("InnerClass").unwrap();
            let inner_class_type = inner_class.cast_into::<PyType>().unwrap();
            assert_eq!(inner_class_type.name().unwrap(), "InnerClass");
            assert_eq!(
                inner_class_type.qualname().unwrap(),
                "OuterClass.InnerClass"
            );
            let module_name = module_name.to_str().unwrap();
            let qualname = format!("{module_name}.OuterClass.InnerClass");
            assert_eq!(inner_class_type.module().unwrap(), module_name);
            assert_eq!(
                inner_class_type.fully_qualified_name().unwrap(),
                qualname.as_str()
            );
        });
    }
}
