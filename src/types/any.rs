use crate::class::basic::CompareOp;
use crate::conversion::{
    AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, PyTryFrom, ToBorrowedObject, ToPyObject,
};
use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::exceptions::PyTypeError;
use crate::type_object::PyTypeObject;
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
    /// Converts this `PyAny` to a concrete Python type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyAny, PyDict, PyList};
    ///
    /// Python::with_gil(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>().unwrap());
    ///     let any: &PyAny = dict.as_ref();
    ///     assert!(any.downcast::<PyDict>().is_ok());
    ///     assert!(any.downcast::<PyList>().is_err());
    /// });
    /// ```
    pub fn downcast<T>(&self) -> Result<&T, PyDowncastError>
    where
        for<'py> T: PyTryFrom<'py>,
    {
        <T as PyTryFrom>::try_from(self)
    }

    /// Determines whether this object has the given attribute.
    ///
    /// This is equivalent to the Python expression `hasattr(self, attr_name)`.
    pub fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    /// Retrieves an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name`.
    pub fn getattr<N>(&self, attr_name: N) -> PyResult<&PyAny>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name = value`.
    pub fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: ToBorrowedObject,
        V: ToBorrowedObject,
    {
        attr_name.with_borrowed_ptr(self.py(), move |attr_name| {
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(
                    self.py(),
                    ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value),
                )
            })
        })
    }

    /// Deletes an attribute.
    ///
    /// This is equivalent to the Python statement `del self.attr_name`.
    pub fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            err::error_on_minusone(self.py(), ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
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
        let py = self.py();
        // Almost the same as ffi::PyObject_RichCompareBool, but this one doesn't try self == other.
        // See https://github.com/PyO3/pyo3/issues/985 for more.
        let do_compare = |other, op| unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_RichCompare(self.as_ptr(), other, op))
                .and_then(|obj| obj.is_true(py))
        };
        other.with_borrowed_ptr(py, |other| {
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
        })
    }

    /// Tests whether two Python objects obey a given [`CompareOp`].
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
            other.with_borrowed_ptr(self.py(), |other| {
                self.py().from_owned_ptr_or_err(ffi::PyObject_RichCompare(
                    self.as_ptr(),
                    other,
                    compare_op as c_int,
                ))
            })
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
    pub fn call(
        &self,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        let args = args.into_py(self.py()).into_ptr();
        let kwargs = kwargs.into_ptr();
        let result = unsafe {
            let return_value = ffi::PyObject_Call(self.as_ptr(), args, kwargs);
            self.py().from_owned_ptr_or_err(return_value)
        };
        unsafe {
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
        }
        result
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
            if #[cfg(Py_3_9)] {
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
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let module = PyModule::import(py, "operator")?;
    ///     let add = module.getattr("add")?;
    ///     let args = (1, 2);
    ///     let value = add.call1(args)?;
    ///     assert_eq!(value.extract::<i32>()?, 3);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the following Python code:
    ///
    /// ```python
    /// from operator import add
    ///
    /// value = add(1,2)
    /// assert value == 3
    /// ```
    pub fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
        self.call(args, None)
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{IntoPyDict, PyList};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let list = PyList::new(py, vec![3, 6, 5, 4, 7]);
    ///     let kwargs = vec![("reverse", true)].into_py_dict(py);
    ///
    ///     list.call_method("sort", (), Some(kwargs))?;
    ///     assert_eq!(list.extract::<Vec<i32>>()?, vec![7, 6, 5, 4, 3]);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the following Python code:
    ///
    /// ```python
    /// my_list = [3, 6, 5, 4, 7]
    /// my_list.sort(reverse = True)
    /// assert my_list == [7, 6, 5, 4, 3]
    /// ```
    pub fn call_method(
        &self,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let py = self.py();
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let args = args.into_py(py).into_ptr();
            let kwargs = kwargs.into_ptr();
            let result_ptr = ffi::PyObject_Call(ptr, args, kwargs);
            let result = py.from_owned_ptr_or_err(result_ptr);
            ffi::Py_DECREF(ptr);
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
            result
        })
    }

    /// Calls a method on the object without arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyFloat;
    /// use std::f64::consts::PI;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let pi = PyFloat::new(py, PI);
    ///     let ratio = pi.call_method0("as_integer_ratio")?;
    ///     let (a, b) = ratio.extract::<(u64, u64)>()?;
    ///     assert_eq!(a, 884_279_719_003_555);
    ///     assert_eq!(b, 281_474_976_710_656);
    ///     Ok(())
    /// })?;
    /// # Ok(())}
    /// ```
    ///
    /// This is equivalent to the following Python code:
    ///
    /// ```python
    /// import math
    ///
    /// a, b = math.pi.as_integer_ratio()
    /// ```
    pub fn call_method0(&self, name: &str) -> PyResult<&PyAny> {
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(Py_LIMITED_API)))] {
                // Optimized path on python 3.9+
                unsafe {
                    let name = name.into_py(self.py());
            self.py().from_owned_ptr_or_err(ffi::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr()))
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
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyList;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let list = PyList::new(py, vec![1, 3, 4]);
    ///     list.call_method1("insert", (1, 2))?;
    ///     assert_eq!(list.extract::<Vec<u8>>()?, [1, 2, 3, 4]);
    ///     Ok(())
    /// })?;
    /// # Ok(()) }
    /// ```
    ///
    /// This is equivalent to the following Python code:
    ///
    /// ```python
    /// list_ = [1,3,4]
    /// list_.insert(1,2)
    /// assert list_ == [1,2,3,4]
    /// ```
    pub fn call_method1(&self, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
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
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets a collection item value.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToBorrowedObject,
        V: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), move |key| {
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(self.py(), ffi::PyObject_SetItem(self.as_ptr(), key, value))
            })
        })
    }

    /// Deletes an item from the collection.
    ///
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            err::error_on_minusone(self.py(), ffi::PyObject_DelItem(self.as_ptr(), key))
        })
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

    /// Casts the PyObject to a concrete Python object type.
    ///
    /// This can cast only to native Python types, not types implemented in Rust.
    pub fn cast_as<'a, D>(&'a self) -> Result<&'a D, PyDowncastError>
    where
        D: PyTryFrom<'a>,
    {
        D::try_from(self)
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around `FromPyObject::extract()`.
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

    /// Checks whether this object is an instance of type `typ`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, typ)`.
    pub fn is_instance(&self, typ: &PyType) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsInstance(self.as_ptr(), typ.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`,
    /// if the type `T` is known at compile time.
    pub fn is_instance_of<T: PyTypeObject>(&self) -> PyResult<bool> {
        self.is_instance(T::type_object(self.py()))
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToBorrowedObject,
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Contains(self.as_ptr(), ptr)
        });
        match r {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        type_object::PyTypeObject,
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
    fn test_any_isinstance() {
        Python::with_gil(|py| {
            let x = 5.to_object(py).into_ref(py);
            assert!(x.is_instance_of::<PyLong>().unwrap());

            let l = vec![x, x].to_object(py).into_ref(py);
            assert!(l.is_instance_of::<PyList>().unwrap());
        });
    }

    #[test]
    fn test_any_isinstance_of() {
        Python::with_gil(|py| {
            let l = vec![1u8, 2].to_object(py).into_ref(py);
            assert!(l.is_instance(PyList::type_object(py)).unwrap());
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
