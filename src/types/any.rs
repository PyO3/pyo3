use crate::call::PyCallArgs;
use crate::class::basic::CompareOp;
use crate::conversion::{FromPyObjectBound, IntoPyObject};
use crate::err::{DowncastError, DowncastIntoError, PyErr, PyResult};
use crate::exceptions::{PyAttributeError, PyTypeError};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::internal::get_slot::TP_DESCR_GET;
use crate::py_result_ext::PyResultExt;
use crate::type_object::{PyTypeCheck, PyTypeInfo};
#[cfg(not(any(PyPy, GraalPy)))]
use crate::types::PySuper;
use crate::types::{PyDict, PyIterator, PyList, PyString, PyType};
use crate::{err, ffi, Borrowed, BoundObject, IntoPyObjectExt, Py, Python};
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::ffi::c_int;
use std::ptr;

/// Represents any Python object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyAny>`][crate::Py] or [`Bound<'py, PyAny>`][Bound].
///
/// For APIs available on all Python objects, see the [`PyAnyMethods`] trait which is implemented for
/// [`Bound<'py, PyAny>`][Bound].
///
/// See
#[doc = concat!("[the guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/types.html#concrete-python-types)")]
/// for an explanation of the different Python object types.
#[repr(transparent)]
pub struct PyAny(UnsafeCell<ffi::PyObject>);

#[allow(non_snake_case)]
// Copied here as the macro does not accept deprecated functions.
// Originally ffi::object::PyObject_Check, but this is not in the Python C API.
fn PyObject_Check(_: *mut ffi::PyObject) -> c_int {
    1
}

pyobject_native_type_info!(
    PyAny,
    pyobject_native_static_type_object!(ffi::PyBaseObject_Type),
    Some("builtins"),
    #checkfunction=PyObject_Check
);

pyobject_native_type_sized!(PyAny, ffi::PyObject);
// We cannot use `pyobject_subclassable_native_type!()` because it cfgs out on `Py_LIMITED_API`.
impl crate::impl_::pyclass::PyClassBaseType for PyAny {
    type LayoutAsBase = crate::impl_::pycell::PyClassObjectBase<ffi::PyObject>;
    type BaseNativeType = PyAny;
    type Initializer = crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
    type PyClassMutability = crate::pycell::impl_::ImmutableClass;
}

/// This trait represents the Python APIs which are usable on all Python objects.
///
/// It is recommended you import this trait via `use pyo3::prelude::*` rather than
/// by importing this trait directly.
#[doc(alias = "PyAny")]
pub trait PyAnyMethods<'py>: crate::sealed::Sealed {
    /// Returns whether `self` and `other` point to the same object. To compare
    /// the equality of two objects (the `==` operator), use [`eq`](PyAnyMethods::eq).
    ///
    /// This is equivalent to the Python expression `self is other`.
    fn is<T: AsRef<Py<PyAny>>>(&self, other: T) -> bool;

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
    /// # use pyo3::{prelude::*, intern};
    /// #
    /// #[pyfunction]
    /// fn has_version(sys: &Bound<'_, PyModule>) -> PyResult<bool> {
    ///     sys.hasattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    has_version(&sys).unwrap();
    /// # });
    /// ```
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPyObject<'py, Target = PyString>;

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
    /// # use pyo3::{prelude::*, intern};
    /// #
    /// #[pyfunction]
    /// fn version<'py>(sys: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyAny>> {
    ///     sys.getattr(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    version(&sys).unwrap();
    /// # });
    /// ```
    fn getattr<N>(&self, attr_name: N) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>;

    /// Retrieves an attribute value optionally.
    ///
    /// This is equivalent to the Python expression `getattr(self, attr_name, None)`, which may
    /// be more efficient in some cases by simply returning `None` if the attribute is not found
    /// instead of raising `AttributeError`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    ///
    /// # Errors
    /// Returns `Err` if an exception other than `AttributeError` is raised during attribute lookup,
    /// such as a `ValueError` from a property or descriptor.
    ///
    /// # Example: Retrieving an optional attribute
    /// ```
    /// # use pyo3::{prelude::*, intern};
    /// #
    /// #[pyfunction]
    /// fn get_version_if_exists<'py>(sys: &Bound<'py, PyModule>) -> PyResult<Option<Bound<'py, PyAny>>> {
    ///     sys.getattr_opt(intern!(sys.py(), "version"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let sys = py.import("sys").unwrap();
    /// #    let version = get_version_if_exists(&sys).unwrap();
    /// #    assert!(version.is_some());
    /// # });
    /// ```
    fn getattr_opt<N>(&self, attr_name: N) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        N: IntoPyObject<'py, Target = PyString>;

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
    /// # use pyo3::{prelude::*, intern};
    /// #
    /// #[pyfunction]
    /// fn set_answer(ob: &Bound<'_, PyAny>) -> PyResult<()> {
    ///     ob.setattr(intern!(ob.py(), "answer"), 42)
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let ob = PyModule::new(py, "empty").unwrap();
    /// #    set_answer(&ob).unwrap();
    /// # });
    /// ```
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>;

    /// Deletes an attribute.
    ///
    /// This is equivalent to the Python statement `del self.attr_name`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`] macro can be used
    /// to intern `attr_name`.
    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>;

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
    /// Python::attach(|py| -> PyResult<()> {
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
    /// Python::attach(|py| -> PyResult<()> {
    ///     let a = PyFloat::new(py, 0_f64);
    ///     let b = PyString::new(py, "zero");
    ///     assert!(a.compare(b).is_err());
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: IntoPyObject<'py>;

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
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let a = 0_u8.into_pyobject(py)?;
    ///     let b = 42_u8.into_pyobject(py)?;
    ///     assert!(a.rich_compare(b, CompareOp::Le)?.is_truthy()?);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes the negative of self.
    ///
    /// Equivalent to the Python expression `-self`.
    fn neg(&self) -> PyResult<Bound<'py, PyAny>>;

    /// Computes the positive of self.
    ///
    /// Equivalent to the Python expression `+self`.
    fn pos(&self) -> PyResult<Bound<'py, PyAny>>;

    /// Computes the absolute of self.
    ///
    /// Equivalent to the Python expression `abs(self)`.
    fn abs(&self) -> PyResult<Bound<'py, PyAny>>;

    /// Computes `~self`.
    fn bitnot(&self) -> PyResult<Bound<'py, PyAny>>;

    /// Tests whether this object is less than another.
    ///
    /// This is equivalent to the Python expression `self < other`.
    fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Tests whether this object is less than or equal to another.
    ///
    /// This is equivalent to the Python expression `self <= other`.
    fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Tests whether this object is equal to another.
    ///
    /// This is equivalent to the Python expression `self == other`.
    fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Tests whether this object is not equal to another.
    ///
    /// This is equivalent to the Python expression `self != other`.
    fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Tests whether this object is greater than another.
    ///
    /// This is equivalent to the Python expression `self > other`.
    fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Tests whether this object is greater than or equal to another.
    ///
    /// This is equivalent to the Python expression `self >= other`.
    fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>;

    /// Computes `self + other`.
    fn add<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self - other`.
    fn sub<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self * other`.
    fn mul<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self @ other`.
    fn matmul<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self / other`.
    fn div<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self // other`.
    fn floor_div<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self % other`.
    fn rem<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `divmod(self, other)`.
    fn divmod<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self << other`.
    fn lshift<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self >> other`.
    fn rshift<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self ** other % modulus` (`pow(self, other, modulus)`).
    /// `py.None()` may be passed for the `modulus`.
    fn pow<O1, O2>(&self, other: O1, modulus: O2) -> PyResult<Bound<'py, PyAny>>
    where
        O1: IntoPyObject<'py>,
        O2: IntoPyObject<'py>;

    /// Computes `self & other`.
    fn bitand<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self | other`.
    fn bitor<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

    /// Computes `self ^ other`.
    fn bitxor<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>;

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
    /// Python::attach(|py| -> PyResult<()> {
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
    /// use pyo3_ffi::c_str;
    /// use std::ffi::CStr;
    ///
    /// const CODE: &CStr = c_str!(r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {"cruel": "world"}
    ///     return "called with args and kwargs"
    /// "#);
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let module = PyModule::from_code(py, CODE, c_str!("func.py"), c_str!(""))?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = fun.call(args, Some(&kwargs))?;
    ///     assert_eq!(result.extract::<String>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call<A>(&self, args: A, kwargs: Option<&Bound<'py, PyDict>>) -> PyResult<Bound<'py, PyAny>>
    where
        A: PyCallArgs<'py>;

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
    /// Python::attach(|py| -> PyResult<()> {
    ///     let module = PyModule::import(py, "builtins")?;
    ///     let help = module.getattr("help")?;
    ///     help.call0()?;
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the Python expression `help()`.
    fn call0(&self) -> PyResult<Bound<'py, PyAny>>;

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3_ffi::c_str;
    /// use std::ffi::CStr;
    ///
    /// const CODE: &CStr = c_str!(r#"
    /// def function(*args, **kwargs):
    ///     assert args == ("hello",)
    ///     assert kwargs == {}
    ///     return "called with args"
    /// "#);
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let module = PyModule::from_code(py, CODE, c_str!("func.py"), c_str!(""))?;
    ///     let fun = module.getattr("function")?;
    ///     let args = ("hello",);
    ///     let result = fun.call1(args)?;
    ///     assert_eq!(result.extract::<String>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call1<A>(&self, args: A) -> PyResult<Bound<'py, PyAny>>
    where
        A: PyCallArgs<'py>;

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
    /// use pyo3_ffi::c_str;
    /// use std::ffi::CStr;
    ///
    /// const CODE: &CStr = c_str!(r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {"cruel": "world"}
    ///         return "called with args and kwargs"
    /// a = A()
    /// "#);
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let module = PyModule::from_code(py, CODE, c_str!("a.py"), c_str!(""))?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let kwargs = PyDict::new(py);
    ///     kwargs.set_item("cruel", "world")?;
    ///     let result = instance.call_method("method", args, Some(&kwargs))?;
    ///     assert_eq!(result.extract::<String>()?, "called with args and kwargs");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method<N, A>(
        &self,
        name: N,
        args: A,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>;

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
    /// use pyo3_ffi::c_str;
    /// use std::ffi::CStr;
    ///
    /// const CODE: &CStr = c_str!(r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ()
    ///         assert kwargs == {}
    ///         return "called with no arguments"
    /// a = A()
    /// "#);
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let module = PyModule::from_code(py, CODE, c_str!("a.py"), c_str!(""))?;
    ///     let instance = module.getattr("a")?;
    ///     let result = instance.call_method0("method")?;
    ///     assert_eq!(result.extract::<String>()?, "called with no arguments");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method0<N>(&self, name: N) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>;

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
    /// use pyo3_ffi::c_str;
    /// use std::ffi::CStr;
    ///
    /// const CODE: &CStr = c_str!(r#"
    /// class A:
    ///     def method(self, *args, **kwargs):
    ///         assert args == ("hello",)
    ///         assert kwargs == {}
    ///         return "called with args"
    /// a = A()
    /// "#);
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let module = PyModule::from_code(py, CODE, c_str!("a.py"), c_str!(""))?;
    ///     let instance = module.getattr("a")?;
    ///     let args = ("hello",);
    ///     let result = instance.call_method1("method", args)?;
    ///     assert_eq!(result.extract::<String>()?, "called with args");
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn call_method1<N, A>(&self, name: N, args: A) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>;

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    fn is_truthy(&self) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    fn is_none(&self) -> bool;

    /// Returns true if the sequence or mapping has a length of 0.
    ///
    /// This is equivalent to the Python expression `len(self) == 0`.
    fn is_empty(&self) -> PyResult<bool>;

    /// Gets an item from the collection.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    fn get_item<K>(&self, key: K) -> PyResult<Bound<'py, PyAny>>
    where
        K: IntoPyObject<'py>;

    /// Sets a collection item value.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
        V: IntoPyObject<'py>;

    /// Deletes an item from the collection.
    ///
    /// This is equivalent to the Python expression `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>;

    /// Takes an object and returns an iterator for it. Returns an error if the object is not
    /// iterable.
    ///
    /// This is typically a new iterator but if the argument is an iterator,
    /// this returns itself.
    ///
    /// # Example: Checking a Python object for iterability
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyAny, PyNone};
    ///
    /// fn is_iterable(obj: &Bound<'_, PyAny>) -> bool {
    ///     match obj.try_iter() {
    ///         Ok(_) => true,
    ///         Err(_) => false,
    ///     }
    /// }
    ///
    /// Python::attach(|py| {
    ///     assert!(is_iterable(&vec![1, 2, 3].into_pyobject(py).unwrap()));
    ///     assert!(!is_iterable(&PyNone::get(py)));
    /// });
    /// ```
    fn try_iter(&self) -> PyResult<Bound<'py, PyIterator>>;

    /// Returns the Python type object for this object's type.
    fn get_type(&self) -> Bound<'py, PyType>;

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
    /// # #![allow(deprecated)]
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>());
    ///     let any = dict.as_any();
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
    /// # #![allow(deprecated)]
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::attach(|py| {
    ///     let class = Py::new(py, Class { i: 0 }).unwrap().into_bound(py).into_any();
    ///
    ///     let class_bound: &Bound<'_, Class> = class.downcast()?;
    ///
    ///     class_bound.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract()?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    // FIXME(icxolu) deprecate in favor of `Bound::cast`
    fn downcast<T>(&self) -> Result<&Bound<'py, T>, DowncastError<'_, 'py>>
    where
        T: PyTypeCheck;

    /// Like `downcast` but takes ownership of `self`.
    ///
    /// In case of an error, it is possible to retrieve `self` again via [`DowncastIntoError::into_inner`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![allow(deprecated)]
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let obj: Bound<'_, PyAny> = PyDict::new(py).into_any();
    ///
    ///     let obj: Bound<'_, PyAny> = match obj.downcast_into::<PyList>() {
    ///         Ok(_) => panic!("obj should not be a list"),
    ///         Err(err) => err.into_inner(),
    ///     };
    ///
    ///     // obj is a dictionary
    ///     assert!(obj.downcast_into::<PyDict>().is_ok());
    /// })
    /// ```
    // FIXME(icxolu) deprecate in favor of `Bound::cast_into`
    fn downcast_into<T>(self) -> Result<Bound<'py, T>, DowncastIntoError<'py>>
    where
        T: PyTypeCheck;

    /// Downcast this `PyAny` to a concrete Python type or pyclass (but not a subclass of it).
    ///
    /// It is almost always better to use [`PyAnyMethods::downcast`] because it accounts for Python
    /// subtyping. Use this method only when you do not want to allow subtypes.
    ///
    /// The advantage of this method over [`PyAnyMethods::downcast`] is that it is faster. The implementation
    /// of `downcast_exact` uses the equivalent of the Python expression `type(self) is T`, whereas
    /// `downcast` uses `isinstance(self, T)`.
    ///
    /// For extracting a Rust-only type, see [`PyAny::extract`](struct.PyAny.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object but not a subtype
    ///
    /// ```rust
    /// # #![allow(deprecated)]
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyBool, PyInt};
    ///
    /// Python::attach(|py| {
    ///     let b = PyBool::new(py, true);
    ///     assert!(b.is_instance_of::<PyBool>());
    ///     let any: &Bound<'_, PyAny> = b.as_any();
    ///
    ///     // `bool` is a subtype of `int`, so `downcast` will accept a `bool` as an `int`
    ///     // but `downcast_exact` will not.
    ///     assert!(any.downcast::<PyInt>().is_ok());
    ///     assert!(any.downcast_exact::<PyInt>().is_err());
    ///
    ///     assert!(any.downcast_exact::<PyBool>().is_ok());
    /// });
    /// ```
    // FIXME(icxolu) deprecate in favor of `Bound::cast_exact`
    fn downcast_exact<T>(&self) -> Result<&Bound<'py, T>, DowncastError<'_, 'py>>
    where
        T: PyTypeInfo;

    /// Like `downcast_exact` but takes ownership of `self`.
    // FIXME(icxolu) deprecate in favor of `Bound::cast_into_exact`
    fn downcast_into_exact<T>(self) -> Result<Bound<'py, T>, DowncastIntoError<'py>>
    where
        T: PyTypeInfo;

    /// Converts this `PyAny` to a concrete Python type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    // FIXME(icxolu) deprecate in favor of `Bound::cast_unchecked`
    unsafe fn downcast_unchecked<T>(&self) -> &Bound<'py, T>;

    /// Like `downcast_unchecked` but takes ownership of `self`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    // FIXME(icxolu) deprecate in favor of `Bound::cast_into_unchecked`
    unsafe fn downcast_into_unchecked<T>(self) -> Bound<'py, T>;

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around
    /// [`FromPyObject::extract_bound()`](crate::FromPyObject::extract_bound).
    fn extract<'a, T>(&'a self) -> PyResult<T>
    where
        T: FromPyObjectBound<'a, 'py>;

    /// Returns the reference count for the Python object.
    fn get_refcnt(&self) -> isize;

    /// Computes the "repr" representation of self.
    ///
    /// This is equivalent to the Python expression `repr(self)`.
    fn repr(&self) -> PyResult<Bound<'py, PyString>>;

    /// Computes the "str" representation of self.
    ///
    /// This is equivalent to the Python expression `str(self)`.
    fn str(&self) -> PyResult<Bound<'py, PyString>>;

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
    fn dir(&self) -> PyResult<Bound<'py, PyList>>;

    /// Checks whether this object is an instance of type `ty`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, ty)`.
    fn is_instance(&self, ty: &Bound<'py, PyAny>) -> PyResult<bool>;

    /// Checks whether this object is an instance of exactly type `ty` (not a subclass).
    ///
    /// This is equivalent to the Python expression `type(self) is ty`.
    fn is_exact_instance(&self, ty: &Bound<'py, PyAny>) -> bool;

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`,
    /// if the type `T` is known at compile time.
    fn is_instance_of<T: PyTypeCheck>(&self) -> bool;

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
        V: IntoPyObject<'py>;

    /// Return a proxy object that delegates method calls to a parent or sibling class of type.
    ///
    /// This is equivalent to the Python expression `super()`
    #[cfg(not(any(PyPy, GraalPy)))]
    fn py_super(&self) -> PyResult<Bound<'py, PySuper>>;
}

macro_rules! implement_binop {
    ($name:ident, $c_api:ident, $op:expr) => {
        #[doc = concat!("Computes `self ", $op, " other`.")]
        fn $name<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
        where
            O: IntoPyObject<'py>,
        {
            fn inner<'py>(
                any: &Bound<'py, PyAny>,
                other: Borrowed<'_, 'py, PyAny>,
            ) -> PyResult<Bound<'py, PyAny>> {
                unsafe { ffi::$c_api(any.as_ptr(), other.as_ptr()).assume_owned_or_err(any.py()) }
            }

            let py = self.py();
            inner(
                self,
                other.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
            )
        }
    };
}

impl<'py> PyAnyMethods<'py> for Bound<'py, PyAny> {
    #[inline]
    fn is<T: AsRef<Py<PyAny>>>(&self, other: T) -> bool {
        ptr::eq(self.as_ptr(), other.as_ref().as_ptr())
    }

    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        // PyObject_HasAttr suppresses all exceptions, which was the behaviour of `hasattr` in Python 2.
        // Use an implementation which suppresses only AttributeError, which is consistent with `hasattr` in Python 3.
        fn inner(py: Python<'_>, getattr_result: PyResult<Bound<'_, PyAny>>) -> PyResult<bool> {
            match getattr_result {
                Ok(_) => Ok(true),
                Err(err) if err.is_instance_of::<PyAttributeError>(py) => Ok(false),
                Err(e) => Err(e),
            }
        }

        inner(self.py(), self.getattr(attr_name))
    }

    fn getattr<N>(&self, attr_name: N) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            attr_name: Borrowed<'_, '_, PyString>,
        ) -> PyResult<Bound<'py, PyAny>> {
            unsafe {
                ffi::PyObject_GetAttr(any.as_ptr(), attr_name.as_ptr())
                    .assume_owned_or_err(any.py())
            }
        }

        inner(
            self,
            attr_name
                .into_pyobject(self.py())
                .map_err(Into::into)?
                .as_borrowed(),
        )
    }

    fn getattr_opt<N>(&self, attr_name: N) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            attr_name: Borrowed<'_, 'py, PyString>,
        ) -> PyResult<Option<Bound<'py, PyAny>>> {
            #[cfg(Py_3_13)]
            {
                let mut resp_ptr: *mut ffi::PyObject = std::ptr::null_mut();
                match unsafe {
                    ffi::PyObject_GetOptionalAttr(any.as_ptr(), attr_name.as_ptr(), &mut resp_ptr)
                } {
                    // Attribute found, result is a new strong reference
                    1 => {
                        let bound = unsafe { Bound::from_owned_ptr(any.py(), resp_ptr) };
                        Ok(Some(bound))
                    }
                    // Attribute not found, result is NULL
                    0 => Ok(None),

                    // An error occurred (other than AttributeError)
                    _ => Err(PyErr::fetch(any.py())),
                }
            }

            #[cfg(not(Py_3_13))]
            {
                match any.getattr(attr_name) {
                    Ok(bound) => Ok(Some(bound)),
                    Err(err) => {
                        let err_type = err
                            .get_type(any.py())
                            .is(PyType::new::<PyAttributeError>(any.py()));
                        match err_type {
                            true => Ok(None),
                            false => Err(err),
                        }
                    }
                }
            }
        }

        let py = self.py();
        inner(self, attr_name.into_pyobject_or_pyerr(py)?.as_borrowed())
    }

    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        fn inner(
            any: &Bound<'_, PyAny>,
            attr_name: Borrowed<'_, '_, PyString>,
            value: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_SetAttr(any.as_ptr(), attr_name.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            attr_name.into_pyobject_or_pyerr(py)?.as_borrowed(),
            value.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        fn inner(any: &Bound<'_, PyAny>, attr_name: Borrowed<'_, '_, PyString>) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_DelAttr(any.as_ptr(), attr_name.as_ptr())
            })
        }

        let py = self.py();
        inner(self, attr_name.into_pyobject_or_pyerr(py)?.as_borrowed())
    }

    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: IntoPyObject<'py>,
    {
        fn inner(any: &Bound<'_, PyAny>, other: Borrowed<'_, '_, PyAny>) -> PyResult<Ordering> {
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
        inner(
            self,
            other.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            other: Borrowed<'_, 'py, PyAny>,
            compare_op: CompareOp,
        ) -> PyResult<Bound<'py, PyAny>> {
            unsafe {
                ffi::PyObject_RichCompare(any.as_ptr(), other.as_ptr(), compare_op as c_int)
                    .assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(
            self,
            other.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
            compare_op,
        )
    }

    fn neg(&self) -> PyResult<Bound<'py, PyAny>> {
        unsafe { ffi::PyNumber_Negative(self.as_ptr()).assume_owned_or_err(self.py()) }
    }

    fn pos(&self) -> PyResult<Bound<'py, PyAny>> {
        fn inner<'py>(any: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
            unsafe { ffi::PyNumber_Positive(any.as_ptr()).assume_owned_or_err(any.py()) }
        }

        inner(self)
    }

    fn abs(&self) -> PyResult<Bound<'py, PyAny>> {
        fn inner<'py>(any: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
            unsafe { ffi::PyNumber_Absolute(any.as_ptr()).assume_owned_or_err(any.py()) }
        }

        inner(self)
    }

    fn bitnot(&self) -> PyResult<Bound<'py, PyAny>> {
        fn inner<'py>(any: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
            unsafe { ffi::PyNumber_Invert(any.as_ptr()).assume_owned_or_err(any.py()) }
        }

        inner(self)
    }

    fn lt<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Lt)
            .and_then(|any| any.is_truthy())
    }

    fn le<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Le)
            .and_then(|any| any.is_truthy())
    }

    fn eq<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Eq)
            .and_then(|any| any.is_truthy())
    }

    fn ne<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Ne)
            .and_then(|any| any.is_truthy())
    }

    fn gt<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Gt)
            .and_then(|any| any.is_truthy())
    }

    fn ge<O>(&self, other: O) -> PyResult<bool>
    where
        O: IntoPyObject<'py>,
    {
        self.rich_compare(other, CompareOp::Ge)
            .and_then(|any| any.is_truthy())
    }

    implement_binop!(add, PyNumber_Add, "+");
    implement_binop!(sub, PyNumber_Subtract, "-");
    implement_binop!(mul, PyNumber_Multiply, "*");
    implement_binop!(matmul, PyNumber_MatrixMultiply, "@");
    implement_binop!(div, PyNumber_TrueDivide, "/");
    implement_binop!(floor_div, PyNumber_FloorDivide, "//");
    implement_binop!(rem, PyNumber_Remainder, "%");
    implement_binop!(lshift, PyNumber_Lshift, "<<");
    implement_binop!(rshift, PyNumber_Rshift, ">>");
    implement_binop!(bitand, PyNumber_And, "&");
    implement_binop!(bitor, PyNumber_Or, "|");
    implement_binop!(bitxor, PyNumber_Xor, "^");

    /// Computes `divmod(self, other)`.
    fn divmod<O>(&self, other: O) -> PyResult<Bound<'py, PyAny>>
    where
        O: IntoPyObject<'py>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            other: Borrowed<'_, 'py, PyAny>,
        ) -> PyResult<Bound<'py, PyAny>> {
            unsafe {
                ffi::PyNumber_Divmod(any.as_ptr(), other.as_ptr()).assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(
            self,
            other.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    /// Computes `self ** other % modulus` (`pow(self, other, modulus)`).
    /// `py.None()` may be passed for the `modulus`.
    fn pow<O1, O2>(&self, other: O1, modulus: O2) -> PyResult<Bound<'py, PyAny>>
    where
        O1: IntoPyObject<'py>,
        O2: IntoPyObject<'py>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            other: Borrowed<'_, 'py, PyAny>,
            modulus: Borrowed<'_, 'py, PyAny>,
        ) -> PyResult<Bound<'py, PyAny>> {
            unsafe {
                ffi::PyNumber_Power(any.as_ptr(), other.as_ptr(), modulus.as_ptr())
                    .assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(
            self,
            other.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
            modulus.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn is_callable(&self) -> bool {
        unsafe { ffi::PyCallable_Check(self.as_ptr()) != 0 }
    }

    fn call<A>(&self, args: A, kwargs: Option<&Bound<'py, PyDict>>) -> PyResult<Bound<'py, PyAny>>
    where
        A: PyCallArgs<'py>,
    {
        if let Some(kwargs) = kwargs {
            args.call(
                self.as_borrowed(),
                kwargs.as_borrowed(),
                crate::call::private::Token,
            )
        } else {
            args.call_positional(self.as_borrowed(), crate::call::private::Token)
        }
    }

    #[inline]
    fn call0(&self) -> PyResult<Bound<'py, PyAny>> {
        unsafe { ffi::compat::PyObject_CallNoArgs(self.as_ptr()).assume_owned_or_err(self.py()) }
    }

    fn call1<A>(&self, args: A) -> PyResult<Bound<'py, PyAny>>
    where
        A: PyCallArgs<'py>,
    {
        args.call_positional(self.as_borrowed(), crate::call::private::Token)
    }

    #[inline]
    fn call_method<N, A>(
        &self,
        name: N,
        args: A,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>,
    {
        if kwargs.is_none() {
            self.call_method1(name, args)
        } else {
            self.getattr(name)
                .and_then(|method| method.call(args, kwargs))
        }
    }

    #[inline]
    fn call_method0<N>(&self, name: N) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        let py = self.py();
        let name = name.into_pyobject_or_pyerr(py)?.into_bound();
        unsafe {
            ffi::compat::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr())
                .assume_owned_or_err(py)
        }
    }

    fn call_method1<N, A>(&self, name: N, args: A) -> PyResult<Bound<'py, PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>,
    {
        let name = name.into_pyobject_or_pyerr(self.py())?;
        args.call_method_positional(
            self.as_borrowed(),
            name.as_borrowed(),
            crate::call::private::Token,
        )
    }

    fn is_truthy(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        err::error_on_minusone(self.py(), v)?;
        Ok(v != 0)
    }

    #[inline]
    fn is_none(&self) -> bool {
        unsafe { ptr::eq(ffi::Py_None(), self.as_ptr()) }
    }

    fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    fn get_item<K>(&self, key: K) -> PyResult<Bound<'py, PyAny>>
    where
        K: IntoPyObject<'py>,
    {
        fn inner<'py>(
            any: &Bound<'py, PyAny>,
            key: Borrowed<'_, 'py, PyAny>,
        ) -> PyResult<Bound<'py, PyAny>> {
            unsafe {
                ffi::PyObject_GetItem(any.as_ptr(), key.as_ptr()).assume_owned_or_err(any.py())
            }
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
        V: IntoPyObject<'py>,
    {
        fn inner(
            any: &Bound<'_, PyAny>,
            key: Borrowed<'_, '_, PyAny>,
            value: Borrowed<'_, '_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_SetItem(any.as_ptr(), key.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
            value.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
    {
        fn inner(any: &Bound<'_, PyAny>, key: Borrowed<'_, '_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(any.py(), unsafe {
                ffi::PyObject_DelItem(any.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    fn try_iter(&self) -> PyResult<Bound<'py, PyIterator>> {
        PyIterator::from_object(self)
    }

    fn get_type(&self) -> Bound<'py, PyType> {
        unsafe { PyType::from_borrowed_type_ptr(self.py(), ffi::Py_TYPE(self.as_ptr())) }
    }

    #[inline]
    fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(self.as_ptr()) }
    }

    #[inline]
    fn downcast<T>(&self) -> Result<&Bound<'py, T>, DowncastError<'_, 'py>>
    where
        T: PyTypeCheck,
    {
        self.cast()
    }

    #[inline]
    fn downcast_into<T>(self) -> Result<Bound<'py, T>, DowncastIntoError<'py>>
    where
        T: PyTypeCheck,
    {
        self.cast_into()
    }

    #[inline]
    fn downcast_exact<T>(&self) -> Result<&Bound<'py, T>, DowncastError<'_, 'py>>
    where
        T: PyTypeInfo,
    {
        self.cast_exact()
    }

    #[inline]
    fn downcast_into_exact<T>(self) -> Result<Bound<'py, T>, DowncastIntoError<'py>>
    where
        T: PyTypeInfo,
    {
        self.cast_into_exact()
    }

    #[inline]
    unsafe fn downcast_unchecked<T>(&self) -> &Bound<'py, T> {
        unsafe { self.cast_unchecked() }
    }

    #[inline]
    unsafe fn downcast_into_unchecked<T>(self) -> Bound<'py, T> {
        unsafe { self.cast_into_unchecked() }
    }

    fn extract<'a, T>(&'a self) -> PyResult<T>
    where
        T: FromPyObjectBound<'a, 'py>,
    {
        FromPyObjectBound::from_py_object_bound(self.as_borrowed())
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    fn repr(&self) -> PyResult<Bound<'py, PyString>> {
        unsafe {
            ffi::PyObject_Repr(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }

    fn str(&self) -> PyResult<Bound<'py, PyString>> {
        unsafe {
            ffi::PyObject_Str(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
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

    fn dir(&self) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            ffi::PyObject_Dir(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }

    #[inline]
    fn is_instance(&self, ty: &Bound<'py, PyAny>) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsInstance(self.as_ptr(), ty.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    #[inline]
    fn is_exact_instance(&self, ty: &Bound<'py, PyAny>) -> bool {
        self.get_type().is(ty)
    }

    #[inline]
    fn is_instance_of<T: PyTypeCheck>(&self) -> bool {
        T::type_check(self)
    }

    #[inline]
    fn is_exact_instance_of<T: PyTypeInfo>(&self) -> bool {
        T::is_exact_type_of(self)
    }

    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: IntoPyObject<'py>,
    {
        fn inner(any: &Bound<'_, PyAny>, value: Borrowed<'_, '_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySequence_Contains(any.as_ptr(), value.as_ptr()) } {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(PyErr::fetch(any.py())),
            }
        }

        let py = self.py();
        inner(
            self,
            value.into_pyobject_or_pyerr(py)?.into_any().as_borrowed(),
        )
    }

    #[cfg(not(any(PyPy, GraalPy)))]
    fn py_super(&self) -> PyResult<Bound<'py, PySuper>> {
        PySuper::new(&self.get_type(), self)
    }
}

impl<'py> Bound<'py, PyAny> {
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
    #[allow(dead_code)] // Currently only used with num-complex+abi3, so dead without that.
    pub(crate) fn lookup_special<N>(&self, attr_name: N) -> PyResult<Option<Bound<'py, PyAny>>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        let py = self.py();
        let self_type = self.get_type();
        let attr = if let Ok(attr) = self_type.getattr(attr_name) {
            attr
        } else {
            return Ok(None);
        };

        // Manually resolve descriptor protocol. (Faster than going through Python.)
        if let Some(descr_get) = attr.get_type().get_slot(TP_DESCR_GET) {
            // attribute is a descriptor, resolve it
            unsafe {
                descr_get(attr.as_ptr(), self.as_ptr(), self_type.as_ptr())
                    .assume_owned_or_err(py)
                    .map(Some)
            }
        } else {
            Ok(Some(attr))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        basic::CompareOp,
        ffi,
        test_utils::generate_unique_module_name,
        types::{IntoPyDict, PyAny, PyAnyMethods, PyBool, PyInt, PyList, PyModule, PyTypeMethods},
        Bound, BoundObject, IntoPyObject, PyTypeInfo, Python,
    };
    use pyo3_ffi::c_str;
    use std::fmt::Debug;

    #[test]
    fn test_lookup_special() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
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
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();

            let int = crate::intern!(py, "__int__");
            let eval_int =
                |obj: Bound<'_, PyAny>| obj.lookup_special(int)?.unwrap().call0()?.extract::<u32>();

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
    fn test_getattr_opt() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class Test:
    class_str_attribute = "class_string"

    @property
    def error(self):
        raise ValueError("This is an intentional error")
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();

            // Get the class Test
            let class_test = module.getattr_opt("Test").unwrap().unwrap();

            // Test attribute that exist
            let cls_attr_str = class_test
                .getattr_opt("class_str_attribute")
                .unwrap()
                .unwrap();
            assert_eq!(cls_attr_str.extract::<String>().unwrap(), "class_string");

            // Test non-existent attribute
            let do_not_exist = class_test.getattr_opt("doNotExist").unwrap();
            assert!(do_not_exist.is_none());

            // Test error attribute
            let instance = class_test.call0().unwrap();
            let error = instance.getattr_opt("error");
            assert!(error.is_err());
            assert!(error
                .unwrap_err()
                .to_string()
                .contains("This is an intentional error"));
        });
    }

    #[test]
    fn test_call_for_non_existing_method() {
        Python::attach(|py| {
            let a = py.eval(ffi::c_str!("42"), None, None).unwrap();
            a.call_method0("__str__").unwrap(); // ok
            assert!(a.call_method("nonexistent_method", (1,), None).is_err());
            assert!(a.call_method0("nonexistent_method").is_err());
            assert!(a.call_method1("nonexistent_method", (1,)).is_err());
        });
    }

    #[test]
    fn test_call_with_kwargs() {
        Python::attach(|py| {
            let list = vec![3, 6, 5, 4, 7].into_pyobject(py).unwrap();
            let dict = vec![("reverse", true)].into_py_dict(py).unwrap();
            list.call_method("sort", (), Some(&dict)).unwrap();
            assert_eq!(list.extract::<Vec<i32>>().unwrap(), vec![7, 6, 5, 4, 3]);
        });
    }

    #[test]
    fn test_call_method0() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class SimpleClass:
    def foo(self):
        return 42
"#
                ),
                c_str!(file!()),
                &generate_unique_module_name("test_module"),
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
        Python::attach(|py| {
            let obj = py.eval(ffi::c_str!("42"), None, None).unwrap();
            assert_eq!(obj.get_type().as_type_ptr(), obj.get_type_ptr());
        });
    }

    #[test]
    fn test_dir() {
        Python::attach(|py| {
            let obj = py.eval(ffi::c_str!("42"), None, None).unwrap();
            let dir = py
                .eval(ffi::c_str!("dir(42)"), None, None)
                .unwrap()
                .cast_into::<PyList>()
                .unwrap();
            let a = obj
                .dir()
                .unwrap()
                .into_iter()
                .map(|x| x.extract::<String>().unwrap());
            let b = dir.into_iter().map(|x| x.extract::<String>().unwrap());
            assert!(a.eq(b));
        });
    }

    #[test]
    fn test_hasattr() {
        Python::attach(|py| {
            let x = 5i32.into_pyobject(py).unwrap();
            assert!(x.is_instance_of::<PyInt>());

            assert!(x.hasattr("to_bytes").unwrap());
            assert!(!x.hasattr("bbbbbbytes").unwrap());
        })
    }

    #[cfg(feature = "macros")]
    #[test]
    #[allow(unknown_lints, non_local_definitions)]
    fn test_hasattr_error() {
        use crate::exceptions::PyValueError;
        use crate::prelude::*;

        #[pyclass(crate = "crate")]
        struct GetattrFail;

        #[pymethods(crate = "crate")]
        impl GetattrFail {
            fn __getattr__(&self, attr: Py<PyAny>) -> PyResult<Py<PyAny>> {
                Err(PyValueError::new_err(attr))
            }
        }

        Python::attach(|py| {
            let obj = Py::new(py, GetattrFail).unwrap();
            let obj = obj.bind(py).as_any();

            assert!(obj
                .hasattr("foo")
                .unwrap_err()
                .is_instance_of::<PyValueError>(py));
        })
    }

    #[test]
    fn test_nan_eq() {
        Python::attach(|py| {
            let nan = py.eval(ffi::c_str!("float('nan')"), None, None).unwrap();
            assert!(nan.compare(&nan).is_err());
        });
    }

    #[test]
    fn test_any_is_instance_of() {
        Python::attach(|py| {
            let x = 5i32.into_pyobject(py).unwrap();
            assert!(x.is_instance_of::<PyInt>());

            let l = vec![&x, &x].into_pyobject(py).unwrap();
            assert!(l.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_any_is_instance() {
        Python::attach(|py| {
            let l = vec![1i8, 2].into_pyobject(py).unwrap();
            assert!(l.is_instance(&py.get_type::<PyList>()).unwrap());
        });
    }

    #[test]
    fn test_any_is_exact_instance_of() {
        Python::attach(|py| {
            let x = 5i32.into_pyobject(py).unwrap();
            assert!(x.is_exact_instance_of::<PyInt>());

            let t = PyBool::new(py, true);
            assert!(t.is_instance_of::<PyInt>());
            assert!(!t.is_exact_instance_of::<PyInt>());
            assert!(t.is_exact_instance_of::<PyBool>());

            let l = vec![&x, &x].into_pyobject(py).unwrap();
            assert!(l.is_exact_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_any_is_exact_instance() {
        Python::attach(|py| {
            let t = PyBool::new(py, true);
            assert!(t.is_instance(&py.get_type::<PyInt>()).unwrap());
            assert!(!t.is_exact_instance(&py.get_type::<PyInt>()));
            assert!(t.is_exact_instance(&py.get_type::<PyBool>()));
        });
    }

    #[test]
    fn test_any_contains() {
        Python::attach(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.into_pyobject(py).unwrap();

            let bad_needle = 7i32.into_pyobject(py).unwrap();
            assert!(!ob.contains(&bad_needle).unwrap());

            let good_needle = 8i32.into_pyobject(py).unwrap();
            assert!(ob.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.into_pyobject(py).unwrap();
            assert!(ob.contains(&type_coerced_needle).unwrap());

            let n: u32 = 42;
            let bad_haystack = n.into_pyobject(py).unwrap();
            let irrelevant_needle = 0i32.into_pyobject(py).unwrap();
            assert!(bad_haystack.contains(&irrelevant_needle).is_err());
        });
    }

    // This is intentionally not a test, it's a generic function used by the tests below.
    fn test_eq_methods_generic<'a, T>(list: &'a [T])
    where
        T: PartialEq + PartialOrd,
        for<'py> &'a T: IntoPyObject<'py>,
        for<'py> <&'a T as IntoPyObject<'py>>::Error: Debug,
    {
        Python::attach(|py| {
            for a in list {
                for b in list {
                    let a_py = a.into_pyobject(py).unwrap().into_any().into_bound();
                    let b_py = b.into_pyobject(py).unwrap().into_any().into_bound();

                    assert_eq!(
                        a.lt(b),
                        a_py.lt(&b_py).unwrap(),
                        "{} < {} should be {}.",
                        a_py,
                        b_py,
                        a.lt(b)
                    );
                    assert_eq!(
                        a.le(b),
                        a_py.le(&b_py).unwrap(),
                        "{} <= {} should be {}.",
                        a_py,
                        b_py,
                        a.le(b)
                    );
                    assert_eq!(
                        a.eq(b),
                        a_py.eq(&b_py).unwrap(),
                        "{} == {} should be {}.",
                        a_py,
                        b_py,
                        a.eq(b)
                    );
                    assert_eq!(
                        a.ne(b),
                        a_py.ne(&b_py).unwrap(),
                        "{} != {} should be {}.",
                        a_py,
                        b_py,
                        a.ne(b)
                    );
                    assert_eq!(
                        a.gt(b),
                        a_py.gt(&b_py).unwrap(),
                        "{} > {} should be {}.",
                        a_py,
                        b_py,
                        a.gt(b)
                    );
                    assert_eq!(
                        a.ge(b),
                        a_py.ge(&b_py).unwrap(),
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
        test_eq_methods_generic::<i32>(&ints);
    }

    #[test]
    fn test_eq_methods_strings() {
        let strings = ["Let's", "test", "some", "eq", "methods"];
        test_eq_methods_generic::<&str>(&strings);
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
        test_eq_methods_generic::<f64>(&floats);
    }

    #[test]
    fn test_eq_methods_bools() {
        let bools = [true, false];
        test_eq_methods_generic::<bool>(&bools);
    }

    #[test]
    fn test_rich_compare_type_error() {
        Python::attach(|py| {
            let py_int = 1i32.into_pyobject(py).unwrap();
            let py_str = "1".into_pyobject(py).unwrap();

            assert!(py_int.rich_compare(&py_str, CompareOp::Lt).is_err());
            assert!(!py_int
                .rich_compare(py_str, CompareOp::Eq)
                .unwrap()
                .is_truthy()
                .unwrap());
        })
    }

    #[test]
    fn test_is_callable() {
        Python::attach(|py| {
            assert!(PyList::type_object(py).is_callable());

            let not_callable = 5i32.into_pyobject(py).unwrap();
            assert!(!not_callable.is_callable());
        });
    }

    #[test]
    fn test_is_empty() {
        Python::attach(|py| {
            let empty_list = PyList::empty(py).into_any();
            assert!(empty_list.is_empty().unwrap());

            let list = PyList::new(py, vec![1, 2, 3]).unwrap().into_any();
            assert!(!list.is_empty().unwrap());

            let not_container = 5i32.into_pyobject(py).unwrap();
            assert!(not_container.is_empty().is_err());
        });
    }

    #[cfg(feature = "macros")]
    #[test]
    #[allow(unknown_lints, non_local_definitions)]
    fn test_fallible_dir() {
        use crate::exceptions::PyValueError;
        use crate::prelude::*;

        #[pyclass(crate = "crate")]
        struct DirFail;

        #[pymethods(crate = "crate")]
        impl DirFail {
            fn __dir__(&self) -> PyResult<Py<PyAny>> {
                Err(PyValueError::new_err("uh-oh!"))
            }
        }

        Python::attach(|py| {
            let obj = Bound::new(py, DirFail).unwrap();
            assert!(obj.dir().unwrap_err().is_instance_of::<PyValueError>(py));
        })
    }
}
