use libc::c_char;
use std;
use ffi;
use err;
use {Python, PyObject, PyResult, PythonObject, PyErr};
use pyptr::{PyPtr, PythonPointer};

/// FromPyObject is implemented by various types that can be extracted from a python object.
pub trait FromPyObject<'p, 'a> {
    fn from_py_object(s: &'a PyObject<'p>) -> PyResult<'p, Self>;
}

/// ToPyObject is implemented for types that can be converted into a python object.
pub trait ToPyObject<'p> for Sized? {
    //type PointerType : 'p + PythonPointer + Deref<PyObject<'p>> = PyPtr<'p, PyObject<'p>>;
    
    //fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, Self::PointerType>;
    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>>;
}

/// BorrowAsPyObject is implemented for types that can be accessed as a borrowed python object
/// (without having to allocate a temporary python object)
trait BorrowAsPyObject<'p> for Sized? {
    fn as_py_object(&self, py: Python<'p>) -> &PyObject<'p>;
}
// Note: I think BorrowAsPyObject is too restricted to be useful, we might as well use &PyObject.
// On the other hand, we might want to optimize ToPyObject so that it doesn't always return a new
// reference: it could return PyResult<A> with associated type A : 'p + PythonPointer + Deref<PyObject<'p>>.
// Then types that can borrow existing python objects can return A=&'p PyObject<'p>,
// while other types can return A=PyPtr<'p, PyObject<'p>>.

// impl ToPyObject for BorrowAsPyObject
impl <'p, T : BorrowAsPyObject<'p>> ToPyObject<'p> for T {
    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        Ok(PyPtr::new(self.as_py_object(py)))
    }
}

// PyObject, PyModule etc.
// We support all three traits (FromPyObject, ToPyObject, BorrowAsPyObject) for
// borrowed python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.
impl <'p, T : PythonObject<'p>> BorrowAsPyObject<'p> for T {
    #[inline]
    fn as_py_object(&self, _: Python<'p>) -> &PyObject<'p> {
        self.as_object()
    }
}

impl <'p, 'a, T : PythonObject<'p>> FromPyObject<'p, 'a> for &'a T {
    #[inline]
    fn from_py_object(s: &'a PyObject<'p>) -> PyResult<'p, &'a T> {
        s.downcast()
    }
}

// PyPtr<T>
// We support all three traits (FromPyObject, ToPyObject, BorrowAsPyObject) for
// owned python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

impl <'p, T : PythonObject<'p>> BorrowAsPyObject<'p> for PyPtr<'p, T> {
    #[inline]
    fn as_py_object(&self, _: Python<'p>) -> &PyObject<'p> {
        self.as_object()
    }
}

impl <'p, 'a, T : PythonObject<'p>> FromPyObject<'p, 'a> for PyPtr<'p, T> {
    #[inline]
    fn from_py_object(s : &'a PyObject<'p>) -> PyResult<'p, PyPtr<'p, T>> {
        PyPtr::new(s).downcast_into()
    }
}


// bool
// As the bool instances have lifetime 'p, we can implement BorrowAsPyObject, not just ToPyObject
impl <'p> BorrowAsPyObject<'p> for bool {
    #[inline]
    fn as_py_object(&self, py: Python<'p>) -> &PyObject<'p> {
        if *self { py.True() } else { py.False() }
    }
}

impl <'p, 'a> FromPyObject<'p, 'a> for bool {
    fn from_py_object(s: &'a PyObject<'p>) -> PyResult<'p, bool> {
        let py = s.python();
        if s == py.True() {
            Ok(true)
        } else if s == py.False() {
            Ok(false)
        } else {
            unimplemented!()
        }
    }
}

// Strings.
// When converting strings to/from python, we need to copy the string data.
// This means we can implement ToPyObject for str, but FromPyObject only for String.
impl <'p> ToPyObject<'p> for str {
    fn to_py_object(&self, py : Python<'p>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        let ptr : *const c_char = self.as_ptr() as *const _;
        let len : ffi::Py_ssize_t = std::num::from_uint(self.len()).unwrap();
        unsafe {
            use std::ascii::AsciiExt;
            let obj = if self.is_ascii() {
                ffi::PyString_FromStringAndSize(ptr, len)
            } else {
                ffi::PyUnicode_FromStringAndSize(ptr, len)
            };
            err::result_from_owned_ptr(py, obj)
        }
    }
}

impl <'p, 'a> FromPyObject<'p, 'a> for String {
    fn from_py_object(s : &'a PyObject<'p>) -> PyResult<'p, String> {
        string_as_slice(s).map(|buf| String::from_utf8_lossy(buf).to_string())
    }
}

pub fn string_as_slice<'a, 'p>(s : &'a PyObject<'p>) -> PyResult<'p, &'a [u8]> {
    unsafe {
        let mut buffer : *mut c_char = std::mem::uninitialized();
        let mut length : ffi::Py_ssize_t = std::mem::uninitialized();
        if ffi::PyString_AsStringAndSize(s.as_ptr(), &mut buffer, &mut length) == 1 {
            Err(PyErr::fetch(s.python()))
        } else {
            let buffer = buffer as *const u8;
            Ok(std::slice::from_raw_buf(std::mem::copy_lifetime(s, &buffer), length as uint))
        }
    }
}

