use crate::err::{self, PyResult};
use crate::instance::Borrowed;
use crate::types::any::PyAnyMethods;
use crate::types::PyTuple;
#[cfg(feature = "gil-refs")]
use crate::PyNativeType;
use crate::{ffi, Bound, PyAny, PyTypeInfo, Python};
use std::borrow::Cow;
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use std::ffi::CStr;
/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, pyobject_native_static_type_object!(ffi::PyType_Type), #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Creates a new type object.
    #[inline]
    pub fn new_bound<T: PyTypeInfo>(py: Python<'_>) -> Bound<'_, PyType> {
        T::type_object_bound(py)
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
        Borrowed::from_ptr_unchecked(py, p.cast())
            .downcast_unchecked()
            .to_owned()
    }
}

#[cfg(feature = "gil-refs")]
impl PyType {
    /// Deprecated form of [`PyType::new_bound`].
    #[inline]
    #[deprecated(
        since = "0.21.0",
        note = "`PyType::new` will be replaced by `PyType::new_bound` in a future PyO3 version"
    )]
    pub fn new<T: PyTypeInfo>(py: Python<'_>) -> &PyType {
        T::type_object_bound(py).into_gil_ref()
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_borrowed().as_type_ptr()
    }

    /// Deprecated form of [`PyType::from_borrowed_type_ptr`].
    ///
    /// # Safety
    ///
    /// - The pointer must a valid non-null reference to a `PyTypeObject`.
    #[inline]
    #[deprecated(
        since = "0.21.0",
        note = "Use `PyType::from_borrowed_type_ptr` instead"
    )]
    pub unsafe fn from_type_ptr(py: Python<'_>, p: *mut ffi::PyTypeObject) -> &PyType {
        Self::from_borrowed_type_ptr(py, p).into_gil_ref()
    }

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    pub fn qualname(&self) -> PyResult<String> {
        self.as_borrowed().qualname()
    }

    /// Gets the full name, which includes the module, of the `PyType`.
    pub fn name(&self) -> PyResult<Cow<'_, str>> {
        self.as_borrowed().name()
    }

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    pub fn is_subclass(&self, other: &PyAny) -> PyResult<bool> {
        self.as_borrowed().is_subclass(&other.as_borrowed())
    }

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    pub fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo,
    {
        self.as_borrowed().is_subclass_of::<T>()
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

    /// Gets the full name, which includes the module, of the `PyType`.
    fn name(&self) -> PyResult<Cow<'_, str>>;

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    fn qualname(&self) -> PyResult<String>;

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
    fn name(&self) -> PyResult<Cow<'_, str>> {
        Borrowed::from(self).name()
    }

    fn qualname(&self) -> PyResult<String> {
        #[cfg(any(Py_LIMITED_API, PyPy, not(Py_3_11)))]
        let name = self.getattr(intern!(self.py(), "__qualname__"))?.extract();

        #[cfg(not(any(Py_LIMITED_API, PyPy, not(Py_3_11))))]
        let name = {
            use crate::ffi_ptr_ext::FfiPtrExt;
            let obj = unsafe {
                ffi::PyType_GetQualName(self.as_type_ptr()).assume_owned_or_err(self.py())?
            };

            obj.extract()
        };

        name
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
        self.is_subclass(&T::type_object_bound(self.py()))
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
                .downcast_into_unchecked()
        };

        mro
    }

    fn bases(&self) -> Bound<'py, PyTuple> {
        #[cfg(any(Py_LIMITED_API, PyPy))]
        let bases = self
            .getattr(intern!(self.py(), "__bases__"))
            .expect("Cannot get `__bases__` from object.")
            .extract()
            .expect("Cannot convert to Rust object.");

        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        let bases = unsafe {
            use crate::ffi_ptr_ext::FfiPtrExt;
            (*self.as_type_ptr())
                .tp_bases
                .assume_borrowed(self.py())
                .to_owned()
                .downcast_into_unchecked()
        };

        bases
    }
}

impl<'a> Borrowed<'a, '_, PyType> {
    fn name(self) -> PyResult<Cow<'a, str>> {
        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        {
            let ptr = self.as_type_ptr();

            let name = unsafe { CStr::from_ptr((*ptr).tp_name) }.to_str()?;

            #[cfg(Py_3_10)]
            if unsafe { ffi::PyType_HasFeature(ptr, ffi::Py_TPFLAGS_IMMUTABLETYPE) } != 0 {
                return Ok(Cow::Borrowed(name));
            }

            Ok(Cow::Owned(name.to_owned()))
        }

        #[cfg(any(Py_LIMITED_API, PyPy))]
        {
            let module = self.getattr(intern!(self.py(), "__module__"))?;

            #[cfg(not(Py_3_11))]
            let name = self.getattr(intern!(self.py(), "__name__"))?;

            #[cfg(Py_3_11)]
            let name = {
                use crate::ffi_ptr_ext::FfiPtrExt;
                unsafe { ffi::PyType_GetName(self.as_type_ptr()).assume_owned_or_err(self.py())? }
            };

            Ok(Cow::Owned(format!("{}.{}", module, name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyBool, PyInt, PyLong, PyTuple, PyTypeMethods};
    use crate::PyAny;
    use crate::Python;

    #[test]
    fn test_type_is_subclass() {
        Python::with_gil(|py| {
            let bool_type = py.get_type_bound::<PyBool>();
            let long_type = py.get_type_bound::<PyLong>();
            assert!(bool_type.is_subclass(&long_type).unwrap());
        });
    }

    #[test]
    fn test_type_is_subclass_of() {
        Python::with_gil(|py| {
            assert!(py
                .get_type_bound::<PyBool>()
                .is_subclass_of::<PyLong>()
                .unwrap());
        });
    }

    #[test]
    fn test_mro() {
        Python::with_gil(|py| {
            assert!(py
                .get_type_bound::<PyBool>()
                .mro()
                .eq(PyTuple::new_bound(
                    py,
                    [
                        py.get_type_bound::<PyBool>(),
                        py.get_type_bound::<PyInt>(),
                        py.get_type_bound::<PyAny>()
                    ]
                ))
                .unwrap());
        });
    }

    #[test]
    fn test_bases_bool() {
        Python::with_gil(|py| {
            assert!(py
                .get_type_bound::<PyBool>()
                .bases()
                .eq(PyTuple::new_bound(py, [py.get_type_bound::<PyInt>()]))
                .unwrap());
        });
    }

    #[test]
    fn test_bases_object() {
        Python::with_gil(|py| {
            assert!(py
                .get_type_bound::<PyAny>()
                .bases()
                .eq(PyTuple::empty_bound(py))
                .unwrap());
        });
    }
}
