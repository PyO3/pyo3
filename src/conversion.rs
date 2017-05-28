use ffi;
use err::PyResult;
use pyptr::{Py, PyPtr};
use python::{Python, ToPythonPointer};
use objects::{PyObject, PyTuple};
use typeob::{PyTypeInfo};


/// Conversion trait that allows various objects to be converted into PyObject
pub trait ToPyObject {

    /// Converts self into a Python object.
    fn to_object<'p>(&self, py: Python<'p>) -> PyPtr<PyObject>;

    /// Converts self into a Python object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    ///
    /// May be more efficient than `to_py_object` because it does not need
    /// to touch any reference counts when the input object already is a Python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        let obj = self.to_object(py).into_object();
        f(obj.as_ptr())
    }
}

pub trait IntoPyObject {

    /// Converts self into a Python object. (Consumes self)
    #[inline]
    fn into_object(self, py: Python) -> PyPtr<PyObject>
        where Self: Sized;
}


/// Conversion trait that allows various objects to be converted into PyTuple object.
pub trait ToPyTuple {

    /// Converts self into a PyTuple object.
    fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyPtr<PyTuple>;

    /// Converts self into a PyTuple object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        let obj = self.to_py_tuple(py);
        f(obj.as_ptr())
    }
}


/// FromPyObject is implemented by various types that can be extracted from a Python object.
///
/// Normal usage is through the `PyObject::extract` helper method:
/// ```let obj: PyObject = ...;
/// let value = try!(obj.extract::<TargetType>(py));
/// ```
///
/// TODO: update this documentation
/// Note: depending on the implementation, the lifetime of the extracted result may
/// depend on the lifetime of the `obj` or the `prepared` variable.
///
/// For example, when extracting `&str` from a python byte string, the resulting string slice will
/// point to the existing string data (lifetime: `'source`).
/// On the other hand, when extracting `&str` from a python unicode string, the preparation step
/// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
/// Since only which of these cases applies depends on the runtime type of the python object,
/// both the `obj` and `prepared` variables must outlive the resulting string slice.
///
/// In cases where the result does not depend on the `'prepared` lifetime,
/// the inherent method `PyObject::extract()` can be used.
pub trait FromPyObject<'source> : Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract<S>(py: &'source Py<'source, S>) -> PyResult<Self>
        where S: PyTypeInfo;
}

pub trait RefFromPyObject<'p> {
    fn with_extracted<F, R>(obj: &'p Py<'p, PyObject>, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R;
}

impl <'p, T: ?Sized> RefFromPyObject<'p> for T
    where for<'a> &'a T: FromPyObject<'p> + Sized
{
    #[inline]
    fn with_extracted<F, R>(obj: &'p Py<'p, PyObject>, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R
    {
        match FromPyObject::extract(obj) {
            Ok(val) => Ok(f(val)),
            Err(e) => Err(e)
        }
    }
}

// Default IntoPyObject implementation
impl<T> IntoPyObject for T where T: ToPyObject
{
    #[inline]
    default fn into_object(self, py: Python) -> PyPtr<PyObject> where Self: Sized
    {
        self.to_object(py)
    }
}

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
// ToPyObject for references
impl <'a, T: ?Sized> ToPyObject for &'a T where T: ToPyObject {

    #[inline]
    default fn to_object(&self, py: Python) -> PyPtr<PyObject> {
        <T as ToPyObject>::to_object(*self, py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        <T as ToPyObject>::with_borrowed_ptr(*self, py, f)
    }
}

/// `Option::Some<T>` is converted like `T`.
/// `Option::None` is converted to Python `None`.
impl <T> ToPyObject for Option<T> where T: ToPyObject {

    fn to_object(&self, py: Python) -> PyPtr<PyObject> {
        match *self {
            Some(ref val) => val.to_object(py),
            None => py.None()
        }
    }
}

impl<T> IntoPyObject for Option<T> where T: IntoPyObject {

    fn into_object(self, py: Python) -> PyPtr<PyObject> {
        match self {
            Some(val) => val.into_object(py),
            None => py.None()
        }
    }
}


/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python) -> PyPtr<PyObject> {
        py.None()
    }
}


impl <'source, T> FromPyObject<'source> for Option<T> where T: FromPyObject<'source> {
    fn extract<S>(obj: &'source Py<'source, S>) -> PyResult<Self>
        where S: PyTypeInfo
    {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            match T::extract(obj) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e)
            }
        }
    }
}
