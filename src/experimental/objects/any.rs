use crate::class::basic::CompareOp;
use crate::conversion::{AsPyPointer, IntoPy, ToBorrowedObject, ToPyObject};
use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::exceptions::PyTypeError;
use crate::objects::{FromPyObject, PyNativeObject, PyTryFrom};
use crate::objects::{PyDict, PyIterator, PyList, PyStr, PyType};
use crate::type_object::PyTypeObject;
use crate::types::{Any, Tuple};
use crate::{err, ffi, Py, PyObject, Python};
use libc::c_int;
use std::cmp::Ordering;

#[repr(transparent)]
pub struct PyAny<'py>(pub(crate) PyObject, pub(crate) Python<'py>);

pyo3_native_object_base!(PyAny<'py>, Any, 'py);

impl PartialEq for PyAny<'_> {
    #[inline]
    fn eq(&self, o: &PyAny) -> bool {
        self.as_ptr() == o.as_ptr()
    }
}

impl AsPyPointer for PyAny<'_> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl Clone for PyAny<'_> {
    fn clone(&self) -> Self {
        Self(self.0.clone_ref(self.1), self.1)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyAny<'py> {
    fn extract(any: &PyAny<'py>) -> PyResult<Self> {
        Ok(any.clone())
    }
}

// unsafe impl crate::PyNativeType for PyAny {}
// unsafe impl crate::type_object::PyLayout<PyAny> for ffi::PyObject {}
// impl crate::type_object::PySizedLayout<PyAny> for ffi::PyObject {}

// pyobject_native_type_convert!(
//     PyAny,
//     ffi::PyObject,
//     ffi::PyBaseObject_Type,
//     Some("builtins"),
//     ffi::PyObject_Check
// );

// pyobject_native_type_extract!(PyAny);

// pyobject_native_type_fmt!(PyAny);

impl<'py> PyAny<'py> {
    /// Convert this PyAny to a concrete Python type.
    pub fn downcast<'a, T>(&'a self) -> Result<&'a T, PyDowncastError>
    where
        T: PyTryFrom<'a, 'py>,
    {
        <T as PyTryFrom>::try_from(self)
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around `FromPyObject::extract()`.
    pub fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a, 'py>,
    {
        FromPyObject::extract(self)
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
    pub fn getattr<N>(&self, attr_name: N) -> PyResult<Self>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            Self::from_raw_or_fetch_err(self.py(), ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
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
    /// This is equivalent to the Python expression `del self.attr_name`.
    pub fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            err::error_on_minusone(self.py(), ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

    /// Compares two Python objects.
    ///
    /// This is equivalent to:
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

    /// Compares two Python objects.
    ///
    /// Depending on the value of `compare_op`, this is equivalent to one of the
    /// following Python expressions:
    ///   * CompareOp::Eq: `self == other`
    ///   * CompareOp::Ne: `self != other`
    ///   * CompareOp::Lt: `self < other`
    ///   * CompareOp::Le: `self <= other`
    ///   * CompareOp::Gt: `self > other`
    ///   * CompareOp::Ge: `self >= other`
    pub fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<Self>
    where
        O: ToPyObject,
    {
        unsafe {
            let result = other.with_borrowed_ptr(self.py(), |other| {
                ffi::PyObject_RichCompare(self.as_ptr(), other, compare_op as c_int)
            });
            Self::from_raw_or_fetch_err(self.py(), result)
        }
    }

    /// Determines whether this object is callable.
    pub fn is_callable(&self) -> bool {
        unsafe { ffi::PyCallable_Check(self.as_ptr()) != 0 }
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    pub fn call(&self, args: impl IntoPy<Py<Tuple>>, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let args = args.into_py(self.py());
        let kwargs_ptr = kwargs.map_or(std::ptr::null_mut(), |dict| dict.as_ptr());
        unsafe {
            let result = ffi::PyObject_Call(self.as_ptr(), args.as_ptr(), kwargs_ptr);
            Self::from_raw_or_fetch_err(self.py(), result)
        }
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    pub fn call0(&self) -> PyResult<Self> {
        self.call((), None)
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    pub fn call1(&self, args: impl IntoPy<Py<Tuple>>) -> PyResult<Self> {
        self.call(args, None)
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// # Example
    /// ```rust
    /// # use pyo3::experimental::prelude::*;
    /// use pyo3::experimental::objects::{IntoPyDict, PyNativeObject};
    ///
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let list = vec![3, 6, 5, 4, 7].to_object(py);
    /// let dict = vec![("reverse", true)].into_py_dict(py);
    /// list.call_method(py, "sort", (), Some(dict.as_owned_ref())).unwrap();
    /// assert_eq!(list.extract::<Vec<i32>>(py).unwrap(), vec![7, 6, 5, 4, 3]);
    ///
    /// let new_element = 1.to_object(py);
    /// list.call_method(py, "append", (new_element,), None).unwrap();
    /// assert_eq!(list.extract::<Vec<i32>>(py).unwrap(), vec![7, 6, 5, 4, 3, 1]);
    /// ```
    pub fn call_method(
        &self,
        name: &str,
        args: impl IntoPy<Py<Tuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Self> {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let py = self.py();
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let args = args.into_py(self.py());
            let kwargs_ptr = kwargs.map_or(std::ptr::null_mut(), |dict| dict.as_ptr());
            let result_ptr = ffi::PyObject_Call(ptr, args.as_ptr(), kwargs_ptr);
            let result = Self::from_raw_or_fetch_err(self.py(), result_ptr);
            ffi::Py_DECREF(ptr);
            result
        })
    }

    /// Calls a method on the object without arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    pub fn call_method0(&self, name: &str) -> PyResult<Self> {
        self.call_method(name, (), None)
    }

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    pub fn call_method1(&self, name: &str, args: impl IntoPy<Py<Tuple>>) -> PyResult<Self> {
        self.call_method(name, args, None)
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    pub fn is_true(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v != 0)
        }
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
    pub fn get_item<K>(&self, key: K) -> PyResult<Self>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            Self::from_raw_or_fetch_err(self.py(), ffi::PyObject_GetItem(self.as_ptr(), key))
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
    pub fn iter(&self) -> PyResult<PyIterator<'py>> {
        PyIterator::from_object(self.py(), self)
    }

    /// Returns the Python type object for this object's type.
    pub fn get_type(&self) -> PyType<'py> {
        unsafe {
            PyType(PyAny::from_borrowed_ptr_or_panic(
                self.py(),
                ffi::Py_TYPE(self.as_ptr()) as _,
            ))
        }
    }

    /// Returns the Python type pointer for this object.
    #[inline]
    pub fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(self.as_ptr()) }
    }

    /// Returns the reference count for the Python object.
    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    /// Computes the "repr" representation of self.
    ///
    /// This is equivalent to the Python expression `repr(self)`.
    pub fn repr(&self) -> PyResult<PyStr<'py>> {
        unsafe {
            Self::from_raw_or_fetch_err(self.py(), ffi::PyObject_Repr(self.as_ptr())).map(PyStr)
        }
    }

    /// Computes the "str" representation of self.
    ///
    /// This is equivalent to the Python expression `str(self)`.
    pub fn str(&self) -> PyResult<PyStr<'py>> {
        unsafe {
            Self::from_raw_or_fetch_err(self.py(), ffi::PyObject_Str(self.as_ptr())).map(PyStr)
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
    pub fn dir(&self) -> PyList<'py> {
        unsafe {
            PyList(Self::from_raw_or_panic(
                self.py(),
                ffi::PyObject_Dir(self.as_ptr()),
            ))
        }
    }

    /// Checks whether this object is an instance of type `T`.
    ///
    /// This is equivalent to the Python expression `isinstance(self, T)`.
    pub fn is_instance<T: PyTypeObject>(&self) -> PyResult<bool> {
        T::type_object(self.py()).is_instance(self)
    }

    pub(crate) fn from_type_any(any: &&'py Any) -> &'py Self {

        unsafe { &*(any as *const &'py Any as *const Self) }
    }

    /// CONVERSION FUNCTIONS

    #[inline]
    pub(crate) unsafe fn from_raw(py: Python<'py>, ptr: *mut ffi::PyObject) -> Option<Self> {
        Py::from_owned_ptr_or_opt(py, ptr).map(|obj| Self(obj, py))
    }

    // Creates a PyOwned without checking the type.
    #[inline]
    pub(crate) unsafe fn from_raw_or_fetch_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Self::from_raw(py, ptr).ok_or_else(|| PyErr::fetch(py))
    }

    #[inline]
    pub(crate) unsafe fn from_raw_or_panic(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(Py::from_owned_ptr(py, ptr), py)
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        Py::from_borrowed_ptr_or_opt(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr_or_fetch_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Py::from_borrowed_ptr_or_err(py, ptr).map(|obj| Self(obj, py))
    }

    #[inline]
    pub(crate) unsafe fn from_borrowed_ptr_or_panic(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Self {
        Self(Py::from_borrowed_ptr(py, ptr), py)
    }
}

#[cfg(test)]
mod test {
    use crate::experimental::ToPyObject;
    use crate::objects::IntoPyDict;
    use crate::types::{Int, List};
    use crate::Python;

    #[test]
    fn test_call_for_non_existing_method() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let a = py.eval("42", None, None).unwrap();
        a.call_method0("__str__").unwrap(); // ok
        assert!(a.call_method("nonexistent_method", (1,), None).is_err());
        assert!(a.call_method0("nonexistent_method").is_err());
        assert!(a.call_method1("nonexistent_method", (1,)).is_err());
    }

    #[test]
    fn test_call_with_kwargs() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = vec![3, 6, 5, 4, 7].to_object(py);
        let dict = vec![("reverse", true)].into_py_dict(py);
        list.call_method("sort", (), Some(&dict)).unwrap();
        assert_eq!(list.extract::<Vec<i32>>().unwrap(), vec![7, 6, 5, 4, 3]);
    }

    #[test]
    fn test_type() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = py.eval("42", None, None).unwrap();
        assert_eq!(obj.get_type().as_type_ptr(), obj.get_type_ptr())
    }

    #[test]
    fn test_dir() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = py.eval("42", None, None).unwrap();
        let dir = py
            .eval("dir(42)", None, None)
            .unwrap()
            .downcast::<List>()
            .unwrap()
            .to_owned();
        let a = obj
            .dir()
            .into_iter()
            .map(|x| x.extract::<String>().unwrap());
        let b = dir.into_iter().map(|x| x.extract::<String>().unwrap());
        assert!(a.eq(b));
    }

    #[test]
    fn test_nan_eq() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let nan = py.eval("float('nan')", None, None).unwrap();
        assert!(nan.compare(nan).is_err());
    }

    #[test]
    fn test_any_isinstance() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let x = 5.to_object(py);
        assert!(x.is_instance::<Int>().unwrap());

        let l = vec![&x, &x].to_object(py);
        assert!(l.is_instance::<List>().unwrap());
    }
}
