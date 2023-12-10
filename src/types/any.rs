use crate::class::basic::CompareOp;
use crate::conversion::{AsPyPointer, FromPyObject, IntoPy, ToPyObject};
use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::exceptions::{PyAttributeError, PyTypeError};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Py2;
use crate::py_result_ext::PyResultExt;
use crate::type_object::{HasPyGilRef, PyTypeCheck, PyTypeInfo};
#[cfg(not(PyPy))]
use crate::types::PySuper;
use crate::types::{PyDict, PyIterator, PyList, PyString, PyTuple, PyType};
use crate::{err, ffi, Py, PyNativeType, Python};
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::os::raw::c_int;

/// Represents any Python object.
///
/// It currently only appears as a *reference*, `&PyAny`,
/// with a lifetime that represents the scope during which the GIL is held.
///
/// `PyAny` has some interesting properties, which it shares
/// with the other [native Python types](crate::types):
///
/// - It can only be obtained and used while the GIL is held,
/// therefore its API does not require a [`Python<'py>`](crate::Python) token.
/// - It can't be used in situations where the GIL is temporarily released,
/// such as [`Python::allow_threads`](crate::Python::allow_threads)'s closure.
/// - The underlying Python object, if mutable, can be mutated through any reference.
/// - It can be converted to the GIL-independent [`Py`]`<`[`PyAny`]`>`,
/// allowing it to outlive the GIL scope. However, using [`Py`]`<`[`PyAny`]`>`'s API
/// *does* require a [`Python<'py>`](crate::Python) token.
///
/// It can be cast to a concrete type with PyAny::downcast (for native Python types only)
/// and FromPyObject::extract. See their documentation for more information.
///
/// See [the guide](https://pyo3.rs/latest/types.html) for an explanation
/// of the different Python object types.
#[repr(transparent)]
pub struct PyAny(UnsafeCell<ffi::PyObject>);

unsafe impl AsPyPointer for PyAny {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.get()
    }
}

#[allow(non_snake_case)]
// Copied here as the macro does not accept deprecated functions.
// Originally ffi::object::PyObject_Check, but this is not in the Python C API.
fn PyObject_Check(_: *mut ffi::PyObject) -> c_int {
    1
}

pyobject_native_type_base!(PyAny);

pyobject_native_type_info!(
    PyAny,
    pyobject_native_static_type_object!(ffi::PyBaseObject_Type),
    Some("builtins"),
    #checkfunction=PyObject_Check
);

pyobject_native_type_extract!(PyAny);

pyobject_native_type_sized!(PyAny, ffi::PyObject);

impl PyAny {
    /// Returns whether `self` and `other` point to the same object. To compare
    /// the equality of two objects (the `==` operator), use [`eq`](PyAny::eq).
    ///
    /// This is equivalent to the Python expression `self is other`.
    #[inline]
    pub fn is<T: AsPyPointer>(&self, other: &T) -> bool {
        Py2::borrowed_from_gil_ref(&self).is(other)
    }

    /// Determines whether this object has the given attribute.
    ///
    /// This is equivalent to the Python expression `hasattr(self, attr_name)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn has_version(sys: &PyModule) -> PyResult<bool> {
    ///     sys.hasattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    has_version(sys).unwrap();
    /// # });
    /// ```
    pub fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPy<Py<PyString>>,
    {
        Py2::borrowed_from_gil_ref(&self).hasattr(attr_name)
    }

    /// Retrieves an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, PyAny, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn version(sys: &PyModule) -> PyResult<&PyAny> {
    ///     sys.getattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    version(sys).unwrap();
    /// # });
    /// ```
    pub fn getattr<N>(&self, attr_name: N) -> PyResult<&PyAny>
    where
        N: IntoPy<Py<PyString>>,
    {
        Py2::borrowed_from_gil_ref(&self)
            .getattr(attr_name)
            .map(Py2::into_gil_ref)
    }

    /// Retrieve an attribute value, skipping the instance dictionary during the lookup but still
    /// binding the object to the instance.
    ///
    /// This is useful when trying to resolve Python's "magic" methods like `__getitem__`, which
    /// are looked up starting from the type object.  This returns an `Option` as it is not
    /// typically a direct error for the special lookup to fail, as magic methods are optional in
    /// many situations in which they might be called.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    pub(crate) fn lookup_special<N>(&self, attr_name: N) -> PyResult<Option<&PyAny>>
    where
        N: IntoPy<Py<PyString>>,
    {
        let py = self.py();
        let self_type = self.get_type();
        let attr = if let Ok(attr) = self_type.getattr(attr_name) {
            attr
        } else {
            return Ok(None);
        };

        // Manually resolve descriptor protocol.
        if cfg!(Py_3_10)
            || unsafe { ffi::PyType_HasFeature(attr.get_type_ptr(), ffi::Py_TPFLAGS_HEAPTYPE) } != 0
        {
            // This is the preferred faster path, but does not work on static types (generally,
            // types defined in extension modules) before Python 3.10.
            unsafe {
                let descr_get_ptr = ffi::PyType_GetSlot(attr.get_type_ptr(), ffi::Py_tp_descr_get);
                if descr_get_ptr.is_null() {
                    return Ok(Some(attr));
                }
                let descr_get: ffi::descrgetfunc = std::mem::transmute(descr_get_ptr);
                let ret = descr_get(attr.as_ptr(), self.as_ptr(), self_type.as_ptr());
                py.from_owned_ptr_or_err(ret).map(Some)
            }
        } else if let Ok(descr_get) = attr.get_type().getattr(crate::intern!(py, "__get__")) {
            descr_get.call1((attr, self, self_type)).map(Some)
        } else {
            Ok(Some(attr))
        }
    }

    /// Sets an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name = value`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, PyAny, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn set_answer(ob: &PyAny) -> PyResult<()> {
    ///     ob.setattr(intern!(ob.py(), "answer"), 42)
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let ob = PyModule::new(py, "empty").unwrap();
    /// #    set_answer(ob).unwrap();
    /// # });
    /// ```
    pub fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).setattr(attr_name, value)
    }

    /// Deletes an attribute.
    ///
    /// This is equivalent to the Python statement `del self.attr_name`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    pub fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
    {
        Py2::borrowed_from_gil_ref(&self).delattr(attr_name)
    }

    /// Returns an [`Ordering`] between `self` and `other`.
    ///
    /// This is equivalent to the following Python code:
    /// ```python
    /// if self == other:
    ///     return Equal
    /// elif a < b:
    ///     return Less
    /// elif a > b:
    ///     return Greater
    /// else:
    ///     raise TypeError("PyAny::compare(): All comparisons returned false")
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyFloat;
    /// use std::cmp::Ordering;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a = PyFloat::new(py, 0_f64);
    ///     let b = PyFloat::new(py, 42_f64);
    ///     assert_eq!(a.compare(b)?, Ordering::Less);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// It will return `PyErr` for values that cannot be compared:
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyFloat, PyString};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a = PyFloat::new(py, 0_f64);
    ///     let b = PyString::new(py, "zero");
    ///     assert!(a.compare(b).is_err());
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    pub fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).compare(other)
    }

    /// Tests whether two Python objects obey a given [`CompareOp`].
    ///
    /// [`lt`](Self::lt), [`le`](Self::le), [`eq`](Self::eq), [`ne`](Self::ne),
    /// [`gt`](Self::gt) and [`ge`](Self::ge) are the specialized versions
    /// of this function.
    ///
    /// Depending on the value of `compare_op`, this is equivalent to one of the
    /// following Python expressions:
    ///
    /// | `compare_op` | Python expression |
    /// | :---: | :----: |
    /// | [`CompareOp::Eq`] | `self == other` |
    /// | [`CompareOp::Ne`] | `self != other` |
    /// | [`CompareOp::Lt`] | `self < other` |
    /// | [`CompareOp::Le`] | `self <= other` |
    /// | [`CompareOp::Gt`] | `self > other` |
    /// | [`CompareOp::Ge`] | `self >= other` |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::class::basic::CompareOp;
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyInt;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a: &PyInt = 0_u8.into_py(py).into_ref(py).downcast()?;
    ///     let b: &PyInt = 42_u8.into_py(py).into_ref(py).downcast()?;
    ///     assert!(a.rich_compare(b, CompareOp::Le)?.is_truthy()?);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    pub fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<&PyAny>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self)
            .rich_compare(other, compare_op)
            .map(Py2::into_gil_ref)
    }

    /// Tests whether this object is less than another.
    ///
    /// This is equivalent to the Python expression `self < other`.
    pub fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).lt(other)
    }

    /// Tests whether this object is less than or equal to another.
    ///
    /// This is equivalent to the Python expression `self <= other`.
    pub fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).le(other)
    }

    /// Tests whether this object is equal to another.
    ///
    /// This is equivalent to the Python expression `self == other`.
    pub fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).eq(other)
    }

    /// Tests whether this object is not equal to another.
    ///
    /// This is equivalent to the Python expression `self != other`.
    pub fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).ne(other)
    }

    /// Tests whether this object is greater than another.
    ///
    /// This is equivalent to the Python expression `self > other`.
    pub fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).gt(other)
    }

    /// Tests whether this object is greater than or equal to another.
    ///
    /// This is equivalent to the Python expression `self >= other`.
    pub fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).ge(other)
    }

    /// Determines whether this object appears callable.
    ///
    /// This is equivalent to Python's [`callable()`][1] function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let builtins = PyModule::import(py, "builtins")?;
    ///     let print = builtins.getattr("print")?;
    ///     assert!(print.is_callable());
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the Python statement `assert callable(print)`.
    ///
    /// Note that unless an API needs to distinguish between callable and
    /// non-callable objects, there is no point in checking for callability.
    /// Instead, it is better to just do the call and handle potential
    /// exceptions.
    ///
    /// [1]: https://docs.python.org/3/library/functions.html#callable
    pub fn is_callable(&self) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_callable()
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// const CODE: &str = r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {"cruel": "world"}
    ///     return "called with args and kwargs"
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = fun.call(args, Some(kwargs))?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn call(
        &self,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        Py2::borrowed_from_gil_ref(&self)
            .call(args, kwargs)
            .map(Py2::into_gil_ref)
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let module = PyModule::import(py, "builtins")?;
    ///     let help = module.getattr("help")?;
    ///     help.call0()?;
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the Python expression `help()`.
    pub fn call0(&self) -> PyResult<&PyAny> {
        Py2::borrowed_from_gil_ref(&self)
            .call0()
            .map(Py2::into_gil_ref)
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {}
    ///     return "called with args"
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let result = fun.call1(args)?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
        Py2::borrowed_from_gil_ref(&self)
            .call1(args)
            .map(Py2::into_gil_ref)
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {"cruel": "world"}
    ///         return "called with args and kwargs"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = instance.call_method("method", args, Some(kwargs))?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn call_method<N, A>(&self, name: N, args: A, kwargs: Option<&PyDict>) -> PyResult<&PyAny>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
    {
        Py2::borrowed_from_gil_ref(&self)
            .call_method(name, args, kwargs)
            .map(Py2::into_gil_ref)
    }

    /// Calls a method on the object without arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ()
    ///         assert kwargs == {}
    ///         return "called with no arguments"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let result = instance.call_method0("method")?;
    ///     assert_eq!(result.extract::<&str>()?, "called with no arguments");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn call_method0<N>(&self, name: N) -> PyResult<&PyAny>
    where
        N: IntoPy<Py<PyString>>,
    {
        Py2::borrowed_from_gil_ref(&self)
            .call_method0(name)
            .map(Py2::into_gil_ref)
    }

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {}
    ///         return "called with args"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let result = instance.call_method1("method", args)?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn call_method1<N, A>(&self, name: N, args: A) -> PyResult<&PyAny>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
    {
        Py2::borrowed_from_gil_ref(&self)
            .call_method1(name, args)
            .map(Py2::into_gil_ref)
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    #[deprecated(since = "0.21.0", note = "use `.is_truthy()` instead")]
    pub fn is_true(&self) -> PyResult<bool> {
        self.is_truthy()
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This applies truth value testing equivalent to the Python expression `bool(self)`.
    pub fn is_truthy(&self) -> PyResult<bool> {
        Py2::borrowed_from_gil_ref(&self).is_truthy()
    }

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    #[inline]
    pub fn is_none(&self) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_none()
    }

    /// Returns whether the object is Ellipsis, e.g. `...`.
    ///
    /// This is equivalent to the Python expression `self is ...`.
    #[deprecated(since = "0.20.0", note = "use `.is(py.Ellipsis())` instead")]
    pub fn is_ellipsis(&self) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_ellipsis()
    }

    /// Returns true if the sequence or mapping has a length of 0.
    ///
    /// This is equivalent to the Python expression `len(self) == 0`.
    pub fn is_empty(&self) -> PyResult<bool> {
        Py2::borrowed_from_gil_ref(&self).is_empty()
    }

    /// Gets an item from the collection.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    pub fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self)
            .get_item(key)
            .map(Py2::into_gil_ref)
    }

    /// Sets a collection item value.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).set_item(key, value)
    }

    /// Deletes an item from the collection.
    ///
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).del_item(key)
    }

    /// Takes an object and returns an iterator for it.
    ///
    /// This is typically a new iterator but if the argument is an iterator,
    /// this returns itself.
    pub fn iter(&self) -> PyResult<&PyIterator> {
        Py2::borrowed_from_gil_ref(&self).iter().map(|py2| {
            // Can't use into_gil_ref here because T: PyTypeInfo bound is not satisfied
            // Safety: into_ptr produces a valid pointer to PyIterator object
            unsafe { self.py().from_owned_ptr(py2.into_ptr()) }
        })
    }

    /// Returns the Python type object for this object's type.
    pub fn get_type(&self) -> &PyType {
        Py2::borrowed_from_gil_ref(&self).get_type()
    }

    /// Returns the Python type pointer for this object.
    #[inline]
    pub fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        Py2::borrowed_from_gil_ref(&self).get_type_ptr()
    }

    /// Downcast this `PyAny` to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid downcasting yourself by just specifying
    /// the desired type in function or method signatures.
    /// However, manual downcasting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`PyAny::extract`](struct.PyAny.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::with_gil(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>());
    ///     let any: &PyAny = dict.as_ref();
    ///
    ///     assert!(any.downcast::<PyDict>().is_ok());
    ///     assert!(any.downcast::<PyList>().is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `PyObject` that
    /// might actually be a pyclass.
    ///
    /// ```rust
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::with_gil(|py| {
    ///     let class: &PyAny = Py::new(py, Class { i: 0 }).unwrap().into_ref(py);
    ///
    ///     let class_cell: &PyCell<Class> = class.downcast()?;
    ///
    ///     class_cell.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract()?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn downcast<T>(&self) -> Result<&T, PyDowncastError<'_>>
    where
        T: PyTypeCheck<AsRefTarget = T>,
    {
        if T::type_check(self) {
            // Safety: type_check is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(PyDowncastError::new(self, T::NAME))
        }
    }

    /// Downcast this `PyAny` to a concrete Python type or pyclass (but not a subclass of it).
    ///
    /// It is almost always better to use [`PyAny::downcast`] because it accounts for Python
    /// subtyping. Use this method only when you do not want to allow subtypes.
    ///
    /// The advantage of this method over [`PyAny::downcast`] is that it is faster. The implementation
    /// of `downcast_exact` uses the equivalent of the Python expression `type(self) is T`, whereas
    /// `downcast` uses `isinstance(self, T)`.
    ///
    /// For extracting a Rust-only type, see [`PyAny::extract`](struct.PyAny.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object but not a subtype
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyBool, PyLong};
    ///
    /// Python::with_gil(|py| {
    ///     let b = PyBool::new(py, true);
    ///     assert!(b.is_instance_of::<PyBool>());
    ///     let any: &PyAny = b.as_ref();
    ///
    ///     // `bool` is a subtype of `int`, so `downcast` will accept a `bool` as an `int`
    ///     // but `downcast_exact` will not.
    ///     assert!(any.downcast::<PyLong>().is_ok());
    ///     assert!(any.downcast_exact::<PyLong>().is_err());
    ///
    ///     assert!(any.downcast_exact::<PyBool>().is_ok());
    /// });
    /// ```
    #[inline]
    pub fn downcast_exact<T>(&self) -> Result<&T, PyDowncastError<'_>>
    where
        T: PyTypeInfo<AsRefTarget = T>,
    {
        if T::is_exact_type_of(self) {
            // Safety: type_check is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(PyDowncastError::new(self, T::NAME))
        }
    }

    /// Converts this `PyAny` to a concrete Python type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn downcast_unchecked<T>(&self) -> &T
    where
        T: HasPyGilRef<AsRefTarget = T>,
    {
        &*(self.as_ptr() as *const T)
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around [`FromPyObject::extract()`].
    #[inline]
    pub fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>,
    {
        FromPyObject::extract(self)
    }

    /// Returns the reference count for the Python object.
    pub fn get_refcnt(&self) -> isize {
        Py2::borrowed_from_gil_ref(&self).get_refcnt()
    }

    /// Computes the "repr" representation of self.
    ///
    /// This is equivalent to the Python expression `repr(self)`.
    pub fn repr(&self) -> PyResult<&PyString> {
        Py2::borrowed_from_gil_ref(&self)
            .repr()
            .map(Py2::into_gil_ref)
    }

    /// Computes the "str" representation of self.
    ///
    /// This is equivalent to the Python expression `str(self)`.
    pub fn str(&self) -> PyResult<&PyString> {
        Py2::borrowed_from_gil_ref(&self)
            .str()
            .map(Py2::into_gil_ref)
    }

    /// Retrieves the hash code of self.
    ///
    /// This is equivalent to the Python expression `hash(self)`.
    pub fn hash(&self) -> PyResult<isize> {
        Py2::borrowed_from_gil_ref(&self).hash()
    }

    /// Returns the length of the sequence or mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> PyResult<usize> {
        Py2::borrowed_from_gil_ref(&self).len()
    }

    /// Returns the list of attributes of this object.
    ///
    /// This is equivalent to the Python expression `dir(self)`.
    pub fn dir(&self) -> &PyList {
        Py2::borrowed_from_gil_ref(&self).dir().into_gil_ref()
    }

    /// Checks whether this object is an instance of type `ty`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, ty)`.
    #[inline]
    pub fn is_instance(&self, ty: &PyAny) -> PyResult<bool> {
        Py2::borrowed_from_gil_ref(&self).is_instance(Py2::borrowed_from_gil_ref(&ty))
    }

    /// Checks whether this object is an instance of exactly type `ty` (not a subclass).
    ///
    /// This is equivalent to the Python expression `type(self) is ty`.
    #[inline]
    pub fn is_exact_instance(&self, ty: &PyAny) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_exact_instance(Py2::borrowed_from_gil_ref(&ty))
    }

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`,
    /// if the type `T` is known at compile time.
    #[inline]
    pub fn is_instance_of<T: PyTypeInfo>(&self) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_instance_of::<T>()
    }

    /// Checks whether this object is an instance of exactly type `T`.
    ///
    /// This is equivalent to the Python expression `type(self) is T`,
    /// if the type `T` is known at compile time.
    #[inline]
    pub fn is_exact_instance_of<T: PyTypeInfo>(&self) -> bool {
        Py2::borrowed_from_gil_ref(&self).is_exact_instance_of::<T>()
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).contains(value)
    }

    /// Returns a GIL marker constrained to the lifetime of this type.
    #[inline]
    pub fn py(&self) -> Python<'_> {
        PyNativeType::py(self)
    }

    /// Returns the raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// Callers are responsible for ensuring that the pointer does not outlive self.
    ///
    /// The reference is borrowed; callers should not decrease the reference count
    /// when they are finished with the pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        self as *const PyAny as *mut ffi::PyObject
    }

    /// Returns an owned raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// The reference is owned; when finished the caller should either transfer ownership
    /// of the pointer or decrease the reference count (e.g. with [`pyo3::ffi::Py_DecRef`](crate::ffi::Py_DecRef)).
    #[inline]
    pub fn into_ptr(&self) -> *mut ffi::PyObject {
        // Safety: self.as_ptr() returns a valid non-null pointer
        unsafe { ffi::_Py_NewRef(self.as_ptr()) }
    }

    /// Return a proxy object that delegates method calls to a parent or sibling class of type.
    ///
    /// This is equivalent to the Python expression `super()`
    #[cfg(not(PyPy))]
    pub fn py_super(&self) -> PyResult<&PySuper> {
        Py2::borrowed_from_gil_ref(&self)
            .py_super()
            .map(Py2::into_gil_ref)
    }
}

/// This trait represents the Python APIs which are usable on all Python objects.
///
/// It is recommended you import this trait via `use pyo3::prelude::*` rather than
/// by importing this trait directly.
#[doc(alias = "PyAny")]
pub(crate) trait PyAnyMethods<'py> {
    /// Returns whether `self` and `other` point to the same object. To compare
    /// the equality of two objects (the `==` operator), use [`eq`](PyAny::eq).
    ///
    /// This is equivalent to the Python expression `self is other`.
    fn is<T: AsPyPointer>(&self, other: &T) -> bool;

    /// Determines whether this object has the given attribute.
    ///
    /// This is equivalent to the Python expression `hasattr(self, attr_name)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn has_version(sys: &PyModule) -> PyResult<bool> {
    ///     sys.hasattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    has_version(sys).unwrap();
    /// # });
    /// ```
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPy<Py<PyString>>;

    /// Retrieves an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, PyAny, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn version(sys: &PyModule) -> PyResult<&PyAny> {
    ///     sys.getattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    version(sys).unwrap();
    /// # });
    /// ```
    fn getattr<N>(&self, attr_name: N) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>;

    /// Sets an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name = value`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, PyAny, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn set_answer(ob: &PyAny) -> PyResult<()> {
    ///     ob.setattr(intern!(ob.py(), "answer"), 42)
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let ob = PyModule::new(py, "empty").unwrap();
    /// #    set_answer(ob).unwrap();
    /// # });
    /// ```
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: ToPyObject;

    /// Deletes an attribute.
    ///
    /// This is equivalent to the Python statement `del self.attr_name`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>;

    /// Returns an [`Ordering`] between `self` and `other`.
    ///
    /// This is equivalent to the following Python code:
    /// ```python
    /// if self == other:
    ///     return Equal
    /// elif a < b:
    ///     return Less
    /// elif a > b:
    ///     return Greater
    /// else:
    ///     raise TypeError("PyAny::compare(): All comparisons returned false")
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyFloat;
    /// use std::cmp::Ordering;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a = PyFloat::new(py, 0_f64);
    ///     let b = PyFloat::new(py, 42_f64);
    ///     assert_eq!(a.compare(b)?, Ordering::Less);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// It will return `PyErr` for values that cannot be compared:
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyFloat, PyString};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a = PyFloat::new(py, 0_f64);
    ///     let b = PyString::new(py, "zero");
    ///     assert!(a.compare(b).is_err());
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: ToPyObject;

    /// Tests whether two Python objects obey a given [`CompareOp`].
    ///
    /// [`lt`](Self::lt), [`le`](Self::le), [`eq`](Self::eq), [`ne`](Self::ne),
    /// [`gt`](Self::gt) and [`ge`](Self::ge) are the specialized versions
    /// of this function.
    ///
    /// Depending on the value of `compare_op`, this is equivalent to one of the
    /// following Python expressions:
    ///
    /// | `compare_op` | Python expression |
    /// | :---: | :----: |
    /// | [`CompareOp::Eq`] | `self == other` |
    /// | [`CompareOp::Ne`] | `self != other` |
    /// | [`CompareOp::Lt`] | `self < other` |
    /// | [`CompareOp::Le`] | `self <= other` |
    /// | [`CompareOp::Gt`] | `self > other` |
    /// | [`CompareOp::Ge`] | `self >= other` |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::class::basic::CompareOp;
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyInt;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let a: &PyInt = 0_u8.into_py(py).into_ref(py).downcast()?;
    ///     let b: &PyInt = 42_u8.into_py(py).into_ref(py).downcast()?;
    ///     assert!(a.rich_compare(b, CompareOp::Le)?.is_truthy()?);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<Py2<'py, PyAny>>
    where
        O: ToPyObject;

    /// Tests whether this object is less than another.
    ///
    /// This is equivalent to the Python expression `self < other`.
    fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Tests whether this object is less than or equal to another.
    ///
    /// This is equivalent to the Python expression `self <= other`.
    fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Tests whether this object is equal to another.
    ///
    /// This is equivalent to the Python expression `self == other`.
    fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Tests whether this object is not equal to another.
    ///
    /// This is equivalent to the Python expression `self != other`.
    fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Tests whether this object is greater than another.
    ///
    /// This is equivalent to the Python expression `self > other`.
    fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Tests whether this object is greater than or equal to another.
    ///
    /// This is equivalent to the Python expression `self >= other`.
    fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject;

    /// Determines whether this object appears callable.
    ///
    /// This is equivalent to Python's [`callable()`][1] function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let builtins = PyModule::import(py, "builtins")?;
    ///     let print = builtins.getattr("print")?;
    ///     assert!(print.is_callable());
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the Python statement `assert callable(print)`.
    ///
    /// Note that unless an API needs to distinguish between callable and
    /// non-callable objects, there is no point in checking for callability.
    /// Instead, it is better to just do the call and handle potential
    /// exceptions.
    ///
    /// [1]: https://docs.python.org/3/library/functions.html#callable
    fn is_callable(&self) -> bool;

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// const CODE: &str = r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {"cruel": "world"}
    ///     return "called with args and kwargs"
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = fun.call(args, Some(kwargs))?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call(
        &self,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py2<'py, PyAny>>;

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let module = PyModule::import(py, "builtins")?;
    ///     let help = module.getattr("help")?;
    ///     help.call0()?;
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the Python expression `help()`.
    fn call0(&self) -> PyResult<Py2<'py, PyAny>>;

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {}
    ///     return "called with args"
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let result = fun.call1(args)?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<Py2<'py, PyAny>>;

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {"cruel": "world"}
    ///         return "called with args and kwargs"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = instance.call_method("method", args, Some(kwargs))?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method<N, A>(
        &self,
        name: N,
        args: A,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>;

    /// Calls a method on the object without arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ()
    ///         assert kwargs == {}
    ///         return "called with no arguments"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let result = instance.call_method0("method")?;
    ///     assert_eq!(result.extract::<&str>()?, "called with no arguments");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method0<N>(&self, name: N) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>;

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// const CODE: &str = r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {}
    ///         return "called with args"
    /// a = A()
    /// "#;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let module = PyModule::from_code(py, CODE, "", "")?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let result = instance.call_method1("method", args)?;
    ///     assert_eq!(result.extract::<&str>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method1<N, A>(&self, name: N, args: A) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>;

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    fn is_truthy(&self) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    fn is_none(&self) -> bool;

    /// Returns whether the object is Ellipsis, e.g. `...`.
    ///
    /// This is equivalent to the Python expression `self is ...`.
    fn is_ellipsis(&self) -> bool;

    /// Returns true if the sequence or mapping has a length of 0.
    ///
    /// This is equivalent to the Python expression `len(self) == 0`.
    fn is_empty(&self) -> PyResult<bool>;

    /// Gets an item from the collection.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    fn get_item<K>(&self, key: K) -> PyResult<Py2<'py, PyAny>>
    where
        K: ToPyObject;

    /// Sets a collection item value.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject;

    /// Deletes an item from the collection.
    ///
    /// This is equivalent to the Python expression `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject;

    /// Takes an object and returns an iterator for it.
    ///
    /// This is typically a new iterator but if the argument is an iterator,
    /// this returns itself.
    fn iter(&self) -> PyResult<Py2<'py, PyIterator>>;

    /// Returns the Python type object for this object's type.
    fn get_type(&self) -> &'py PyType;

    /// Returns the Python type pointer for this object.
    fn get_type_ptr(&self) -> *mut ffi::PyTypeObject;

    /// Downcast this `PyAny` to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid downcasting yourself by just specifying
    /// the desired type in function or method signatures.
    /// However, manual downcasting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`PyAny::extract`](struct.PyAny.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::with_gil(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>());
    ///     let any: &PyAny = dict.as_ref();
    ///
    ///     assert!(any.downcast::<PyDict>().is_ok());
    ///     assert!(any.downcast::<PyList>().is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `PyObject` that
    /// might actually be a pyclass.
    ///
    /// ```rust
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::with_gil(|py| {
    ///     let class: &PyAny = Py::new(py, Class { i: 0 }).unwrap().into_ref(py);
    ///
    ///     let class_cell: &PyCell<Class> = class.downcast()?;
    ///
    ///     class_cell.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract()?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn downcast<T>(&self) -> Result<&Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeCheck;

    /// Like `downcast` but takes ownership of `self`.
    fn downcast_into<T>(self) -> Result<Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeCheck;

    /// Downcast this `PyAny` to a concrete Python type or pyclass (but not a subclass of it).
    ///
    /// It is almost always better to use [`PyAny::downcast`] because it accounts for Python
    /// subtyping. Use this method only when you do not want to allow subtypes.
    ///
    /// The advantage of this method over [`PyAny::downcast`] is that it is faster. The implementation
    /// of `downcast_exact` uses the equivalent of the Python expression `type(self) is T`, whereas
    /// `downcast` uses `isinstance(self, T)`.
    ///
    /// For extracting a Rust-only type, see [`PyAny::extract`](struct.PyAny.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object but not a subtype
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyBool, PyLong};
    ///
    /// Python::with_gil(|py| {
    ///     let b = PyBool::new(py, true);
    ///     assert!(b.is_instance_of::<PyBool>());
    ///     let any: &PyAny = b.as_ref();
    ///
    ///     // `bool` is a subtype of `int`, so `downcast` will accept a `bool` as an `int`
    ///     // but `downcast_exact` will not.
    ///     assert!(any.downcast::<PyLong>().is_ok());
    ///     assert!(any.downcast_exact::<PyLong>().is_err());
    ///
    ///     assert!(any.downcast_exact::<PyBool>().is_ok());
    /// });
    /// ```
    fn downcast_exact<T>(&self) -> Result<&Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeInfo;

    /// Like `downcast_exact` but takes ownership of `self`.
    fn downcast_into_exact<T>(self) -> Result<Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeInfo;

    /// Converts this `PyAny` to a concrete Python type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    unsafe fn downcast_unchecked<T>(&self) -> &Py2<'py, T>;

    /// Like `downcast_unchecked` but takes ownership of `self`.
    unsafe fn downcast_into_unchecked<T>(self) -> Py2<'py, T>;

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around [`FromPyObject::extract()`].
    fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>;

    /// Returns the reference count for the Python object.
    fn get_refcnt(&self) -> isize;

    /// Computes the "repr" representation of self.
    ///
    /// This is equivalent to the Python expression `repr(self)`.
    fn repr(&self) -> PyResult<Py2<'py, PyString>>;

    /// Computes the "str" representation of self.
    ///
    /// This is equivalent to the Python expression `str(self)`.
    fn str(&self) -> PyResult<Py2<'py, PyString>>;

    /// Retrieves the hash code of self.
    ///
    /// This is equivalent to the Python expression `hash(self)`.
    fn hash(&self) -> PyResult<isize>;

    /// Returns the length of the sequence or mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> PyResult<usize>;

    /// Returns the list of attributes of this object.
    ///
    /// This is equivalent to the Python expression `dir(self)`.
    fn dir(&self) -> Py2<'py, PyList>;

    /// Checks whether this object is an instance of type `ty`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, ty)`.
    fn is_instance(&self, ty: &Py2<'py, PyAny>) -> PyResult<bool>;

    /// Checks whether this object is an instance of exactly type `ty` (not a subclass).
    ///
    /// This is equivalent to the Python expression `type(self) is ty`.
    fn is_exact_instance(&self, ty: &Py2<'py, PyAny>) -> bool;

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`,
    /// if the type `T` is known at compile time.
    fn is_instance_of<T: PyTypeInfo>(&self) -> bool;

    /// Checks whether this object is an instance of exactly type `T`.
    ///
    /// This is equivalent to the Python expression `type(self) is T`,
    /// if the type `T` is known at compile time.
    fn is_exact_instance_of<T: PyTypeInfo>(&self) -> bool;

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject;

    /// Return a proxy object that delegates method calls to a parent or sibling class of type.
    ///
    /// This is equivalent to the Python expression `super()`
    #[cfg(not(PyPy))]
    fn py_super(&self) -> PyResult<Py2<'py, PySuper>>;
}

impl<'py> PyAnyMethods<'py> for Py2<'py, PyAny> {
    #[inline]
    fn is<T: AsPyPointer>(&self, other: &T) -> bool {
        self.as_ptr() == other.as_ptr()
    }

    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPy<Py<PyString>>,
    {
        // PyObject_HasAttr suppresses all exceptions, which was the behaviour of `hasattr` in Python 2.
        // Use an implementation which suppresses only AttributeError, which is consistent with `hasattr` in Python 3.
        fn inner(py: Python<'_>, getattr_result: PyResult<Py2<'_, PyAny>>) -> PyResult<bool> {
            match getattr_result {
                Ok(_) => Ok(true),
                Err(err) if err.is_instance_of::<PyAttributeError>(py) => Ok(false),
                Err(e) => Err(e),
            }
        }

        inner(self.py(), self.getattr(attr_name))
    }

    fn getattr<N>(&self, attr_name: N) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
    {
        fn inner<'py>(
            any: &Py2<'py, PyAny>,
            attr_name: Py2<'_, PyString>,
        ) -> PyResult<Py2<'py, PyAny>> {
            unsafe {
                ffi::PyObject_GetAttr(any.as_ptr(), attr_name.as_ptr())
                    .assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(self, attr_name.into_py(self.py()).attach_into(py))
    }

    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: ToPyObject,
    {
        fn inner(
            any: &Py2<'_, PyAny>,
            attr_name: Py2<'_, PyString>,
            value: Py2<'_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_SetAttr(any.as_ptr(), attr_name.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            attr_name.into_py(py).attach_into(py),
            value.to_object(py).attach_into(py),
        )
    }

    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
    {
        fn inner(any: &Py2<'_, PyAny>, attr_name: Py2<'_, PyString>) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_DelAttr(any.as_ptr(), attr_name.as_ptr())
            })
        }

        let py = self.py();
        inner(self, attr_name.into_py(py).attach_into(py))
    }

    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: ToPyObject,
    {
        fn inner(any: &Py2<'_, PyAny>, other: Py2<'_, PyAny>) -> PyResult<Ordering> {
            let other = other.as_ptr();
            // Almost the same as ffi::PyObject_RichCompareBool, but this one doesn't try self == other.
            // See https://github.com/PyO3/pyo3/issues/985 for more.
            let do_compare = |other, op| unsafe {
                ffi::PyObject_RichCompare(any.as_ptr(), other, op)
                    .assume_owned_or_err(any.py())
                    .and_then(|obj| obj.is_truthy())
            };
            if do_compare(other, ffi::Py_EQ)? {
                Ok(Ordering::Equal)
            } else if do_compare(other, ffi::Py_LT)? {
                Ok(Ordering::Less)
            } else if do_compare(other, ffi::Py_GT)? {
                Ok(Ordering::Greater)
            } else {
                Err(PyTypeError::new_err(
                    "PyAny::compare(): All comparisons returned false",
                ))
            }
        }

        let py = self.py();
        inner(self, other.to_object(py).attach_into(py))
    }

    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<Py2<'py, PyAny>>
    where
        O: ToPyObject,
    {
        fn inner<'py>(
            any: &Py2<'py, PyAny>,
            other: Py2<'_, PyAny>,
            compare_op: CompareOp,
        ) -> PyResult<Py2<'py, PyAny>> {
            unsafe {
                ffi::PyObject_RichCompare(any.as_ptr(), other.as_ptr(), compare_op as c_int)
                    .assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(self, other.to_object(py).attach_into(py), compare_op)
    }

    fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Lt)
            .and_then(|any| any.is_truthy())
    }

    fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Le)
            .and_then(|any| any.is_truthy())
    }

    fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Eq)
            .and_then(|any| any.is_truthy())
    }

    fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Ne)
            .and_then(|any| any.is_truthy())
    }

    fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Gt)
            .and_then(|any| any.is_truthy())
    }

    fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Ge)
            .and_then(|any| any.is_truthy())
    }

    fn is_callable(&self) -> bool {
        unsafe { ffi::PyCallable_Check(self.as_ptr()) != 0 }
    }

    fn call(
        &self,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py2<'py, PyAny>> {
        fn inner<'py>(
            any: &Py2<'py, PyAny>,
            args: Py2<'_, PyTuple>,
            kwargs: Option<&PyDict>,
        ) -> PyResult<Py2<'py, PyAny>> {
            unsafe {
                ffi::PyObject_Call(
                    any.as_ptr(),
                    args.as_ptr(),
                    kwargs.map_or(std::ptr::null_mut(), |dict| dict.as_ptr()),
                )
                .assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(self, args.into_py(py).attach_into(py), kwargs)
    }

    fn call0(&self) -> PyResult<Py2<'py, PyAny>> {
        cfg_if::cfg_if! {
            if #[cfg(all(
                not(PyPy),
                any(Py_3_10, all(not(Py_LIMITED_API), Py_3_9)) // PyObject_CallNoArgs was added to python in 3.9 but to limited API in 3.10
            ))] {
                // Optimized path on python 3.9+
                unsafe {
                    ffi::PyObject_CallNoArgs(self.as_ptr()).assume_owned_or_err(self.py())
                }
            } else {
                self.call((), None)
            }
        }
    }

    fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<Py2<'py, PyAny>> {
        self.call(args, None)
    }

    fn call_method<N, A>(
        &self,
        name: N,
        args: A,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
    {
        self.getattr(name)
            .and_then(|method| method.call(args, kwargs))
    }

    fn call_method0<N>(&self, name: N) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
    {
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(any(Py_LIMITED_API, PyPy))))] {
                let py = self.py();

                // Optimized path on python 3.9+
                unsafe {
                    let name = name.into_py(py).attach_into(py);
                    ffi::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr()).assume_owned_or_err(py)
                }
            } else {
                self.call_method(name, (), None)
            }
        }
    }

    fn call_method1<N, A>(&self, name: N, args: A) -> PyResult<Py2<'py, PyAny>>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
    {
        self.call_method(name, args, None)
    }

    fn is_truthy(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        err::error_on_minusone(self.py(), v)?;
        Ok(v != 0)
    }

    #[inline]
    fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    fn is_ellipsis(&self) -> bool {
        unsafe { ffi::Py_Ellipsis() == self.as_ptr() }
    }

    fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    fn get_item<K>(&self, key: K) -> PyResult<Py2<'py, PyAny>>
    where
        K: ToPyObject,
    {
        fn inner<'py>(any: &Py2<'py, PyAny>, key: Py2<'_, PyAny>) -> PyResult<Py2<'py, PyAny>> {
            unsafe {
                ffi::PyObject_GetItem(any.as_ptr(), key.as_ptr()).assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).attach_into(py))
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        fn inner(any: &Py2<'_, PyAny>, key: Py2<'_, PyAny>, value: Py2<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_SetItem(any.as_ptr(), key.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.to_object(py).attach_into(py),
            value.to_object(py).attach_into(py),
        )
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(any: &Py2<'_, PyAny>, key: Py2<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_DelItem(any.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(self, key.to_object(py).attach_into(py))
    }

    fn iter(&self) -> PyResult<Py2<'py, PyIterator>> {
        PyIterator::from_object2(self)
    }

    fn get_type(&self) -> &'py PyType {
        unsafe { PyType::from_type_ptr(self.py(), ffi::Py_TYPE(self.as_ptr())) }
    }

    #[inline]
    fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(self.as_ptr()) }
    }

    #[inline]
    fn downcast<T>(&self) -> Result<&Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeCheck,
    {
        if T::type_check(self.as_gil_ref()) {
            // Safety: type_check is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(PyDowncastError::new(self.clone().into_gil_ref(), T::NAME))
        }
    }

    #[inline]
    fn downcast_into<T>(self) -> Result<Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeCheck,
    {
        if T::type_check(self.as_gil_ref()) {
            // Safety: type_check is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_into_unchecked() })
        } else {
            Err(PyDowncastError::new(self.clone().into_gil_ref(), T::NAME))
        }
    }

    #[inline]
    fn downcast_exact<T>(&self) -> Result<&Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeInfo,
    {
        if self.is_exact_instance_of::<T>() {
            // Safety: is_exact_instance_of is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(PyDowncastError::new(self.clone().into_gil_ref(), T::NAME))
        }
    }

    #[inline]
    fn downcast_into_exact<T>(self) -> Result<Py2<'py, T>, PyDowncastError<'py>>
    where
        T: PyTypeInfo,
    {
        if self.is_exact_instance_of::<T>() {
            // Safety: is_exact_instance_of is responsible for ensuring that the type is correct
            Ok(unsafe { self.downcast_into_unchecked() })
        } else {
            Err(PyDowncastError::new(self.into_gil_ref(), T::NAME))
        }
    }

    #[inline]
    unsafe fn downcast_unchecked<T>(&self) -> &Py2<'py, T> {
        &*(self as *const Py2<'py, PyAny>).cast()
    }

    #[inline]
    unsafe fn downcast_into_unchecked<T>(self) -> Py2<'py, T> {
        std::mem::transmute(self)
    }

    fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>,
    {
        FromPyObject::extract(self.as_gil_ref())
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    fn repr(&self) -> PyResult<Py2<'py, PyString>> {
        unsafe {
            ffi::PyObject_Repr(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn str(&self) -> PyResult<Py2<'py, PyString>> {
        unsafe {
            ffi::PyObject_Str(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    fn hash(&self) -> PyResult<isize> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        crate::err::error_on_minusone(self.py(), v)?;
        Ok(v)
    }

    fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        crate::err::error_on_minusone(self.py(), v)?;
        Ok(v as usize)
    }

    fn dir(&self) -> Py2<'py, PyList> {
        unsafe {
            ffi::PyObject_Dir(self.as_ptr())
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    #[inline]
    fn is_instance(&self, ty: &Py2<'py, PyAny>) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsInstance(self.as_ptr(), ty.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    #[inline]
    fn is_exact_instance(&self, ty: &Py2<'py, PyAny>) -> bool {
        self.get_type().is(ty)
    }

    #[inline]
    fn is_instance_of<T: PyTypeInfo>(&self) -> bool {
        T::is_type_of(self.as_gil_ref())
    }

    #[inline]
    fn is_exact_instance_of<T: PyTypeInfo>(&self) -> bool {
        T::is_exact_type_of(self.as_gil_ref())
    }

    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        fn inner(any: &Py2<'_, PyAny>, value: Py2<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySequence_Contains(any.as_ptr(), value.as_ptr()) } {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(PyErr::fetch(any.py())),
            }
        }

        let py = self.py();
        inner(self, value.to_object(py).attach_into(py))
    }

    #[cfg(not(PyPy))]
    fn py_super(&self) -> PyResult<Py2<'py, PySuper>> {
        PySuper::new2(Py2::borrowed_from_gil_ref(&self.get_type()), self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        basic::CompareOp,
        types::{IntoPyDict, PyAny, PyBool, PyList, PyLong, PyModule},
        PyTypeInfo, Python, ToPyObject,
    };

    #[test]
    fn test_lookup_special() {
        Python::with_gil(|py| {
            let module = PyModule::from_code(
                py,
                r#"
class CustomCallable:
    def __call__(self):
        return 1

class SimpleInt:
    def __int__(self):
        return 1

class InheritedInt(SimpleInt): pass

class NoInt: pass

class NoDescriptorInt:
    __int__ = CustomCallable()

class InstanceOverrideInt:
    def __int__(self):
        return 1
instance_override = InstanceOverrideInt()
instance_override.__int__ = lambda self: 2

class ErrorInDescriptorInt:
    @property
    def __int__(self):
        raise ValueError("uh-oh!")

class NonHeapNonDescriptorInt:
    # A static-typed callable that doesn't implement `__get__`.  These are pretty hard to come by.
    __int__ = int
                "#,
                "test.py",
                "test",
            )
            .unwrap();

            let int = crate::intern!(py, "__int__");
            let eval_int =
                |obj: &PyAny| obj.lookup_special(int)?.unwrap().call0()?.extract::<u32>();

            let simple = module.getattr("SimpleInt").unwrap().call0().unwrap();
            assert_eq!(eval_int(simple).unwrap(), 1);
            let inherited = module.getattr("InheritedInt").unwrap().call0().unwrap();
            assert_eq!(eval_int(inherited).unwrap(), 1);
            let no_descriptor = module.getattr("NoDescriptorInt").unwrap().call0().unwrap();
            assert_eq!(eval_int(no_descriptor).unwrap(), 1);
            let missing = module.getattr("NoInt").unwrap().call0().unwrap();
            assert!(missing.lookup_special(int).unwrap().is_none());
            // Note the instance override should _not_ call the instance method that returns 2,
            // because that's not how special lookups are meant to work.
            let instance_override = module.getattr("instance_override").unwrap();
            assert_eq!(eval_int(instance_override).unwrap(), 1);
            let descriptor_error = module
                .getattr("ErrorInDescriptorInt")
                .unwrap()
                .call0()
                .unwrap();
            assert!(descriptor_error.lookup_special(int).is_err());
            let nonheap_nondescriptor = module
                .getattr("NonHeapNonDescriptorInt")
                .unwrap()
                .call0()
                .unwrap();
            assert_eq!(eval_int(nonheap_nondescriptor).unwrap(), 0);
        })
    }

    #[test]
    fn test_call_for_non_existing_method() {
        Python::with_gil(|py| {
            let a = py.eval("42", None, None).unwrap();
            a.call_method0("__str__").unwrap(); // ok
            assert!(a.call_method("nonexistent_method", (1,), None).is_err());
            assert!(a.call_method0("nonexistent_method").is_err());
            assert!(a.call_method1("nonexistent_method", (1,)).is_err());
        });
    }

    #[test]
    fn test_call_with_kwargs() {
        Python::with_gil(|py| {
            let list = vec![3, 6, 5, 4, 7].to_object(py);
            let dict = vec![("reverse", true)].into_py_dict(py);
            list.call_method(py, "sort", (), Some(dict)).unwrap();
            assert_eq!(list.extract::<Vec<i32>>(py).unwrap(), vec![7, 6, 5, 4, 3]);
        });
    }

    #[test]
    fn test_call_method0() {
        Python::with_gil(|py| {
            let module = PyModule::from_code(
                py,
                r#"
class SimpleClass:
    def foo(self):
        return 42
"#,
                file!(),
                "test_module",
            )
            .expect("module creation failed");

            let simple_class = module.getattr("SimpleClass").unwrap().call0().unwrap();
            assert_eq!(
                simple_class
                    .call_method0("foo")
                    .unwrap()
                    .extract::<u32>()
                    .unwrap(),
                42
            );
        })
    }

    #[test]
    fn test_type() {
        Python::with_gil(|py| {
            let obj = py.eval("42", None, None).unwrap();
            assert_eq!(obj.get_type().as_type_ptr(), obj.get_type_ptr());
        });
    }

    #[test]
    fn test_dir() {
        Python::with_gil(|py| {
            let obj = py.eval("42", None, None).unwrap();
            let dir = py
                .eval("dir(42)", None, None)
                .unwrap()
                .downcast::<PyList>()
                .unwrap();
            let a = obj
                .dir()
                .into_iter()
                .map(|x| x.extract::<String>().unwrap());
            let b = dir.into_iter().map(|x| x.extract::<String>().unwrap());
            assert!(a.eq(b));
        });
    }

    #[test]
    fn test_hasattr() {
        Python::with_gil(|py| {
            let x = 5.to_object(py).into_ref(py);
            assert!(x.is_instance_of::<PyLong>());

            assert!(x.hasattr("to_bytes").unwrap());
            assert!(!x.hasattr("bbbbbbytes").unwrap());
        })
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_hasattr_error() {
        use crate::exceptions::PyValueError;
        use crate::prelude::*;

        #[pyclass(crate = "crate")]
        struct GetattrFail;

        #[pymethods(crate = "crate")]
        impl GetattrFail {
            fn __getattr__(&self, attr: PyObject) -> PyResult<PyObject> {
                Err(PyValueError::new_err(attr))
            }
        }

        Python::with_gil(|py| {
            let obj = Py::new(py, GetattrFail).unwrap();
            let obj = obj.as_ref(py).as_ref();

            assert!(obj
                .hasattr("foo")
                .unwrap_err()
                .is_instance_of::<PyValueError>(py));
        })
    }

    #[test]
    fn test_nan_eq() {
        Python::with_gil(|py| {
            let nan = py.eval("float('nan')", None, None).unwrap();
            assert!(nan.compare(nan).is_err());
        });
    }

    #[test]
    fn test_any_is_instance_of() {
        Python::with_gil(|py| {
            let x = 5.to_object(py).into_ref(py);
            assert!(x.is_instance_of::<PyLong>());

            let l = vec![x, x].to_object(py).into_ref(py);
            assert!(l.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_any_is_instance() {
        Python::with_gil(|py| {
            let l = vec![1u8, 2].to_object(py).into_ref(py);
            assert!(l.is_instance(py.get_type::<PyList>()).unwrap());
        });
    }

    #[test]
    fn test_any_is_exact_instance_of() {
        Python::with_gil(|py| {
            let x = 5.to_object(py).into_ref(py);
            assert!(x.is_exact_instance_of::<PyLong>());

            let t = PyBool::new(py, true);
            assert!(t.is_instance_of::<PyLong>());
            assert!(!t.is_exact_instance_of::<PyLong>());
            assert!(t.is_exact_instance_of::<PyBool>());

            let l = vec![x, x].to_object(py).into_ref(py);
            assert!(l.is_exact_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_any_is_exact_instance() {
        Python::with_gil(|py| {
            let t = PyBool::new(py, true);
            assert!(t.is_instance(py.get_type::<PyLong>()).unwrap());
            assert!(!t.is_exact_instance(py.get_type::<PyLong>()));
            assert!(t.is_exact_instance(py.get_type::<PyBool>()));
        });
    }

    #[test]
    fn test_any_contains() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py).into_ref(py);

            let bad_needle = 7i32.to_object(py);
            assert!(!ob.contains(&bad_needle).unwrap());

            let good_needle = 8i32.to_object(py);
            assert!(ob.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.to_object(py);
            assert!(ob.contains(&type_coerced_needle).unwrap());

            let n: u32 = 42;
            let bad_haystack = n.to_object(py).into_ref(py);
            let irrelevant_needle = 0i32.to_object(py);
            assert!(bad_haystack.contains(&irrelevant_needle).is_err());
        });
    }

    // This is intentionally not a test, it's a generic function used by the tests below.
    fn test_eq_methods_generic<T>(list: &[T])
    where
        T: PartialEq + PartialOrd + ToPyObject,
    {
        Python::with_gil(|py| {
            for a in list {
                for b in list {
                    let a_py = a.to_object(py).into_ref(py);
                    let b_py = b.to_object(py).into_ref(py);

                    assert_eq!(
                        a.lt(b),
                        a_py.lt(b_py).unwrap(),
                        "{} < {} should be {}.",
                        a_py,
                        b_py,
                        a.lt(b)
                    );
                    assert_eq!(
                        a.le(b),
                        a_py.le(b_py).unwrap(),
                        "{} <= {} should be {}.",
                        a_py,
                        b_py,
                        a.le(b)
                    );
                    assert_eq!(
                        a.eq(b),
                        a_py.eq(b_py).unwrap(),
                        "{} == {} should be {}.",
                        a_py,
                        b_py,
                        a.eq(b)
                    );
                    assert_eq!(
                        a.ne(b),
                        a_py.ne(b_py).unwrap(),
                        "{} != {} should be {}.",
                        a_py,
                        b_py,
                        a.ne(b)
                    );
                    assert_eq!(
                        a.gt(b),
                        a_py.gt(b_py).unwrap(),
                        "{} > {} should be {}.",
                        a_py,
                        b_py,
                        a.gt(b)
                    );
                    assert_eq!(
                        a.ge(b),
                        a_py.ge(b_py).unwrap(),
                        "{} >= {} should be {}.",
                        a_py,
                        b_py,
                        a.ge(b)
                    );
                }
            }
        });
    }

    #[test]
    fn test_eq_methods_integers() {
        let ints = [-4, -4, 1, 2, 0, -100, 1_000_000];
        test_eq_methods_generic(&ints);
    }

    #[test]
    fn test_eq_methods_strings() {
        let strings = ["Let's", "test", "some", "eq", "methods"];
        test_eq_methods_generic(&strings);
    }

    #[test]
    fn test_eq_methods_floats() {
        let floats = [
            -1.0,
            2.5,
            0.0,
            3.0,
            std::f64::consts::PI,
            10.0,
            10.0 / 3.0,
            -1_000_000.0,
        ];
        test_eq_methods_generic(&floats);
    }

    #[test]
    fn test_eq_methods_bools() {
        let bools = [true, false];
        test_eq_methods_generic(&bools);
    }

    #[test]
    fn test_rich_compare_type_error() {
        Python::with_gil(|py| {
            let py_int = 1.to_object(py).into_ref(py);
            let py_str = "1".to_object(py).into_ref(py);

            assert!(py_int.rich_compare(py_str, CompareOp::Lt).is_err());
            assert!(!py_int
                .rich_compare(py_str, CompareOp::Eq)
                .unwrap()
                .is_truthy()
                .unwrap());
        })
    }

    #[test]
    #[allow(deprecated)]
    fn test_is_ellipsis() {
        Python::with_gil(|py| {
            let v = py
                .eval("...", None, None)
                .map_err(|e| e.display(py))
                .unwrap();

            assert!(v.is_ellipsis());

            let not_ellipsis = 5.to_object(py).into_ref(py);
            assert!(!not_ellipsis.is_ellipsis());
        });
    }

    #[test]
    fn test_is_callable() {
        Python::with_gil(|py| {
            assert!(PyList::type_object(py).is_callable());

            let not_callable = 5.to_object(py).into_ref(py);
            assert!(!not_callable.is_callable());
        });
    }

    #[test]
    fn test_is_empty() {
        Python::with_gil(|py| {
            let empty_list: &PyAny = PyList::empty(py);
            assert!(empty_list.is_empty().unwrap());

            let list: &PyAny = PyList::new(py, vec![1, 2, 3]);
            assert!(!list.is_empty().unwrap());

            let not_container = 5.to_object(py).into_ref(py);
            assert!(not_container.is_empty().is_err());
        });
    }
}
