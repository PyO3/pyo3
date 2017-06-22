use ffi;
use err::PyResult;
use python::{Python, ToPyPointer, PyDowncastFrom};
use pointer::PyObjectPtr;
use objects::{PyObject, PyTuple};
use objectprotocol::ObjectProtocol;
use typeob::PyTypeInfo;
use instance::Py;


/// Conversion trait that allows various objects to be converted into `PyObject`
pub trait ToPyObject {

    /// Converts self into a Python object.
    fn to_object(&self, py: Python) -> PyObjectPtr;

    /// Converts self into a Python object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    ///
    /// May be more efficient than `to_py_object` because it does not need
    /// to touch any reference counts when the input object already is a Python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        let obj = self.to_object(py);
        let result = f(obj.as_ptr());
        py.release(obj);
        result
    }
}

pub trait IntoPyObject {

    /// Converts self into a Python object. (Consumes self)
    #[inline]
    fn into_object(self, py: Python) -> PyObjectPtr
        where Self: Sized;
}


/// Conversion trait that allows various objects to be converted into `PyTuple` object.
pub trait IntoPyTuple {

    /// Converts self into a PyTuple object.
    fn into_tuple(self, py: Python) -> Py<PyTuple>;

}


/// `FromPyObject` is implemented by various types that can be extracted from a Python object.
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
    fn extract(ob: &'source PyObject) -> PyResult<Self>;
}

/*pub trait RefFromPyObject {
    fn with_extracted<F, R>(ob: &PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R;
}

impl <T: ?Sized> RefFromPyObject for T
    where for<'a> &'a T: FromPyObject + Sized
{
    #[inline]
    fn with_extracted<F, R>(obj: &PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R
    {
        match FromPyObject::extract(obj) {
            Ok(val) => Ok(f(val)),
            Err(e) => Err(e)
        }
    }
}*/

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
// `ToPyObject` for references
impl <'a, T: ?Sized> ToPyObject for &'a T where T: ToPyObject {

    #[inline]
    fn to_object(&self, py: Python) -> PyObjectPtr {
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

    fn to_object(&self, py: Python) -> PyObjectPtr {
        match *self {
            Some(ref val) => val.to_object(py),
            None => py.None(),
        }
    }
}
impl<T> IntoPyObject for Option<T> where T: IntoPyObject {

    fn into_object(self, py: Python) -> PyObjectPtr {
        match self {
            Some(val) => val.into_object(py),
            None => py.None(),
        }
    }
}

/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python) -> PyObjectPtr {
        py.None()
    }
}
impl IntoPyObject for () {
    fn into_object(self, py: Python) -> PyObjectPtr {
        py.None()
    }
}

/// Extract reference to instance from `PyObject`
impl<'a, T> FromPyObject<'a> for &'a T
    where T: PyTypeInfo + PyDowncastFrom
{
    #[inline]
    default fn extract(ob: &'a PyObject) -> PyResult<&'a T>
    {
        Ok(ob.cast_as()?)
    }
}

impl<'source, T> FromPyObject<'source> for Option<T> where T: FromPyObject<'source>
{
    fn extract(obj: &'source PyObject) -> PyResult<Self>
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
