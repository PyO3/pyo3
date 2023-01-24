use crate::class::basic::CompareOp;
use crate::conversion::{AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, PyTryFrom, ToPyObject};
use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::exceptions::PyTypeError;
use crate::type_object::PyTypeInfo;
#[cfg(not(PyPy))]
use crate::types::PySuper;
use crate::types::{PyDict, PyIterator, PyList, PyString, PyTuple, PyType};
use crate::{err, ffi, Py, PyNativeType, PyObject, Python};
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

impl crate::AsPyPointer for PyAny {
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
    ffi::PyBaseObject_Type,
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
        self.as_ptr() == other.as_ptr()
    }

    /// Determines whether this object has the given attribute.
    ///
    /// This is equivalent to the Python expression `hasattr(self, attr_name)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    pub fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPy<Py<PyString>>,
    {
        let py = self.py();
        let attr_name = attr_name.into_py(py);

        unsafe { Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name.as_ptr()) != 0) }
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
        let py = self.py();
        let attr_name = attr_name.into_py(py);

        unsafe {
            let ret = ffi::PyObject_GetAttr(self.as_ptr(), attr_name.as_ptr());
            py.from_owned_ptr_or_err(ret)
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
        let py = self.py();
        let attr_name = attr_name.into_py(py);
        let value = value.to_object(py);

        unsafe {
            let ret = ffi::PyObject_SetAttr(self.as_ptr(), attr_name.as_ptr(), value.as_ptr());
            err::error_on_minusone(py, ret)
        }
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
        let py = self.py();
        let attr_name = attr_name.into_py(py);

        unsafe {
            let ret = ffi::PyObject_DelAttr(self.as_ptr(), attr_name.as_ptr());
            err::error_on_minusone(py, ret)
        }
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
        self._compare(other.to_object(self.py()))
    }

    fn _compare(&self, other: PyObject) -> PyResult<Ordering> {
        let py = self.py();
        let other = other.as_ptr();
        // Almost the same as ffi::PyObject_RichCompareBool, but this one doesn't try self == other.
        // See https://github.com/PyO3/pyo3/issues/985 for more.
        let do_compare = |other, op| unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_RichCompare(self.as_ptr(), other, op))
                .and_then(|obj| obj.is_true(py))
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
    ///     assert!(a.rich_compare(b, CompareOp::Le)?.is_true()?);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    pub fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<&PyAny>
    where
        O: ToPyObject,
    {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PyObject_RichCompare(
                self.as_ptr(),
                other.to_object(self.py()).as_ptr(),
                compare_op as c_int,
            ))
        }
    }

    /// Tests whether this object is less than another.
    ///
    /// This is equivalent to the Python expression `self < other`.
    pub fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Lt)?.is_true()
    }

    /// Tests whether this object is less than or equal to another.
    ///
    /// This is equivalent to the Python expression `self <= other`.
    pub fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Le)?.is_true()
    }

    /// Tests whether this object is equal to another.
    ///
    /// This is equivalent to the Python expression `self == other`.
    pub fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Eq)?.is_true()
    }

    /// Tests whether this object is not equal to another.
    ///
    /// This is equivalent to the Python expression `self != other`.
    pub fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Ne)?.is_true()
    }

    /// Tests whether this object is greater than another.
    ///
    /// This is equivalent to the Python expression `self > other`.
    pub fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Gt)?.is_true()
    }

    /// Tests whether this object is greater than or equal to another.
    ///
    /// This is equivalent to the Python expression `self >= other`.
    pub fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: ToPyObject,
    {
        self.rich_compare(other, CompareOp::Ge)?.is_true()
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
        unsafe { ffi::PyCallable_Check(self.as_ptr()) != 0 }
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
        let py = self.py();

        let args = args.into_py(py);
        let kwargs = kwargs.into_ptr();

        unsafe {
            let return_value = ffi::PyObject_Call(self.as_ptr(), args.as_ptr(), kwargs);
            let ret = py.from_owned_ptr_or_err(return_value);
            ffi::Py_XDECREF(kwargs);
            ret
        }
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
        cfg_if::cfg_if! {
            if #[cfg(all(
                not(PyPy),
                any(Py_3_10, all(not(Py_LIMITED_API), Py_3_9)) // PyObject_CallNoArgs was added to python in 3.9 but to limited API in 3.10
            ))] {
                // Optimized path on python 3.9+
                unsafe {
                    self.py().from_owned_ptr_or_err(ffi::PyObject_CallNoArgs(self.as_ptr()))
                }
            } else {
                self.call((), None)
            }
        }
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
        self.call(args, None)
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
        let py = self.py();

        let callee = self.getattr(name)?;
        let args: Py<PyTuple> = args.into_py(py);
        let kwargs = kwargs.into_ptr();

        unsafe {
            let result_ptr = ffi::PyObject_Call(callee.as_ptr(), args.as_ptr(), kwargs);
            let result = py.from_owned_ptr_or_err(result_ptr);
            ffi::Py_XDECREF(kwargs);
            result
        }
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
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(any(Py_LIMITED_API, PyPy))))] {
                let py = self.py();

                // Optimized path on python 3.9+
                unsafe {
                    let name: Py<PyString> = name.into_py(py);
                    let ptr = ffi::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr());
                    py.from_owned_ptr_or_err(ptr)
                }
            } else {
                self.call_method(name, (), None)
            }
        }
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
        self.call_method(name, args, None)
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    pub fn is_true(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        err::error_on_minusone(self.py(), v)?;
        Ok(v != 0)
    }

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    pub fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns true if the sequence or mapping has a length of 0.
    ///
    /// This is equivalent to the Python expression `len(self) == 0`.
    pub fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    /// Gets an item from the collection.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    pub fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToPyObject,
    {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PyObject_GetItem(
                self.as_ptr(),
                key.to_object(self.py()).as_ptr(),
            ))
        }
    }

    /// Sets a collection item value.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        let py = self.py();
        unsafe {
            err::error_on_minusone(
                py,
                ffi::PyObject_SetItem(
                    self.as_ptr(),
                    key.to_object(py).as_ptr(),
                    value.to_object(py).as_ptr(),
                ),
            )
        }
    }

    /// Deletes an item from the collection.
    ///
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PyObject_DelItem(self.as_ptr(), key.to_object(self.py()).as_ptr()),
            )
        }
    }

    /// Takes an object and returns an iterator for it.
    ///
    /// This is typically a new iterator but if the argument is an iterator,
    /// this returns itself.
    pub fn iter(&self) -> PyResult<&PyIterator> {
        PyIterator::from_object(self.py(), self)
    }

    /// Returns the Python type object for this object's type.
    pub fn get_type(&self) -> &PyType {
        unsafe { PyType::from_type_ptr(self.py(), ffi::Py_TYPE(self.as_ptr())) }
    }

    /// Returns the Python type pointer for this object.
    #[inline]
    pub fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(self.as_ptr()) }
    }

    /// Converts this `PyAny` to a concrete Python type.
    #[deprecated(since = "0.18.0", note = "use the equivalent .downcast()")]
    pub fn cast_as<'a, D>(&'a self) -> Result<&'a D, PyDowncastError<'_>>
    where
        D: PyTryFrom<'a>,
    {
        self.downcast()
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
    /// use pyo3::types::{PyAny, PyDict, PyList};
    ///
    /// Python::with_gil(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>().unwrap());
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
    /// ```
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
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn downcast<'p, T>(&'p self) -> Result<&'p T, PyDowncastError<'_>>
    where
        T: PyTryFrom<'p>,
    {
        <T as PyTryFrom>::try_from(self)
    }

    /// Converts this `PyAny` to a concrete Python type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn downcast_unchecked<'p, T>(&'p self) -> &'p T
    where
        T: PyTryFrom<'p>,
    {
        <T as PyTryFrom>::try_from_unchecked(self)
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around [`FromPyObject::extract()`].
    pub fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>,
    {
        FromPyObject::extract(self)
    }

    /// Returns the reference count for the Python object.
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    /// Computes the "repr" representation of self.
    ///
    /// This is equivalent to the Python expression `repr(self)`.
    pub fn repr(&self) -> PyResult<&PyString> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_Repr(self.as_ptr()))
        }
    }

    /// Computes the "str" representation of self.
    ///
    /// This is equivalent to the Python expression `str(self)`.
    pub fn str(&self) -> PyResult<&PyString> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_Str(self.as_ptr()))
        }
    }

    /// Retrieves the hash code of self.
    ///
    /// This is equivalent to the Python expression `hash(self)`.
    pub fn hash(&self) -> PyResult<isize> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v)
        }
    }

    /// Returns the length of the sequence or mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v as usize)
        }
    }

    /// Returns the list of attributes of this object.
    ///
    /// This is equivalent to the Python expression `dir(self)`.
    pub fn dir(&self) -> &PyList {
        unsafe { self.py().from_owned_ptr(ffi::PyObject_Dir(self.as_ptr())) }
    }

    /// Checks whether this object is an instance of type `ty`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, ty)`.
    pub fn is_instance(&self, ty: &PyAny) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsInstance(self.as_ptr(), ty.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`,
    /// if the type `T` is known at compile time.
    pub fn is_instance_of<T: PyTypeInfo>(&self) -> PyResult<bool> {
        self.is_instance(T::type_object(self.py()))
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        self._contains(value.to_object(self.py()))
    }

    fn _contains(&self, value: PyObject) -> PyResult<bool> {
        match unsafe { ffi::PySequence_Contains(self.as_ptr(), value.as_ptr()) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::fetch(self.py())),
        }
    }

    /// Returns a GIL marker constrained to the lifetime of this type.
    #[inline]
    pub fn py(&self) -> Python<'_> {
        PyNativeType::py(self)
    }

    /// Return a proxy object that delegates method calls to a parent or sibling class of type.
    ///
    /// This is equivalent to the Python expression `super()`
    #[cfg(not(PyPy))]
    pub fn py_super(&self) -> PyResult<&PySuper> {
        PySuper::new(self.get_type(), self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::{IntoPyDict, PyList, PyLong, PyModule},
        Python, ToPyObject,
    };
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
    fn test_nan_eq() {
        Python::with_gil(|py| {
            let nan = py.eval("float('nan')", None, None).unwrap();
            assert!(nan.compare(nan).is_err());
        });
    }

    #[test]
    fn test_any_isinstance_of() {
        Python::with_gil(|py| {
            let x = 5.to_object(py).into_ref(py);
            assert!(x.is_instance_of::<PyLong>().unwrap());

            let l = vec![x, x].to_object(py).into_ref(py);
            assert!(l.is_instance_of::<PyList>().unwrap());
        });
    }

    #[test]
    fn test_any_isinstance() {
        Python::with_gil(|py| {
            let l = vec![1u8, 2].to_object(py).into_ref(py);
            assert!(l.is_instance(py.get_type::<PyList>()).unwrap());
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
}
