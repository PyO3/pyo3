use libc::c_char;
use std;
use ffi;
use err;
use {Python, PyObject, PyResult, PythonObject, PyErr};
use pyptr::{PyPtr, PythonPointer};

/// ToPyObject is implemented for types that can be converted into a python object.
/// The goal is to allow methods that take a python object to take anything that
/// can be converted into a python object.
/// For example, compare calling the following method signatures:
///   fn m1(o: &PyObject) {}
///   fn m2<O>(o: &O) where O : ToPyObject {}
///
///   let o: &PyObject = ...;
///   m1(o);
///   m2(o);
///
///   let p: PyPtr<PyObject> = ...;
///   m1(*p)
///   m2(p)
///
///   let i: i32 = ...;
///   m1(*try!(i.to_py_object(py)))
///   m2(i)
pub trait ToPyObject<'p, 's> for Sized? {
    // The returned pointer type is flexible:
    // it can be either &PyObject or PyPtr<PyObject>, depending on whether
    // the conversion is allocating a new object.
    // This lets us avoid a useless IncRef/DecRef pair
    type PointerType : PythonPointer + std::ops::Deref //<Target = PyObject<'p>>
        = PyPtr<'p, PyObject<'p>>;
    
    fn to_py_object(&'s self, py: Python<'p>) -> PyResult<'p, Self::PointerType>;
    
    // Note that there are 6 cases with methods taking ToPyObject:
    // 1) input is &PyObject, FFI function steals pointer
    //   -> ToPyObject is no-op, PythonPointer::steal_ptr() calls Py_IncRef()
    // 2) input is &PyObject, FFI function borrows pointer
    //   -> ToPyObject is no-op, PythonPointer::as_ptr() also is no-op
    // 3) input is &PyPtr<PyObject>, FFI function steals pointer
    //   -> ToPyObject borrows content, PythonPointer::steal_ptr() calls Py_IncRef()
    //    Not optimal, we'd prefer to take the input PyPtr by value.
    // 4) input is &PyPtr<PyObject>, FFI function borrows pointer
    //   -> ToPyObject borrows content, PythonPointer::as_ptr() is no-op
    // 5) input is &str, int, etc., FFI function steals pointer
    //   -> ToPyObject allocates new object, PythonPointer::steal_ptr() grabs existing owned pointer
    // 6) input is &str, int, etc., FFI function borrows pointer
    //   -> ToPyObject allocates new object, PythonPointer::as_ptr() is no-op,
    //      PyPtr::drop calls Py_DecRef()

    // So the only non-optimal case (3) is the one stealing from a PyPtr<PyObject>,
    // which is unavoidable as long as to_py_object takes &self.
    // Note that changing ToPyObject to take self by value would cause the PyPtr to become
    // unusable in case (4) as well. Users would have to add a .clone() call if the PyPtr
    // is still needed after the call, making case (4) non-optimal.
    // We could potentially fix this by using separate ToPyObject and IntoPyObject traits
    // for the borrowing and stealing cases.
    
    // Note that the 'PointerType' associated type is essential to avoid unnecessarily
    // touching the reference count in cases (2) and (4).
}

/// FromPyObject is implemented by various types that can be extracted from a python object.
pub trait FromPyObject<'p, 's> {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, Self>;
}

// PyObject, PyModule etc.
// We support FromPyObject and ToPyObject for borrowed python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

impl <'p, 's, T : PythonObject<'p>> ToPyObject<'p, 's> for T {
    type PointerType = &'s PyObject<'p>;
    
    fn to_py_object(&'s self, py: Python<'p>) -> PyResult<'p, &'s PyObject<'p>> {
        Ok(self.as_object())
    }
}

impl <'p, 's, T : PythonObject<'p>> FromPyObject<'p, 's> for &'s T {
    #[inline]
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, &'s T> {
        s.downcast()
    }
}

// PyPtr<T>
// We support all three traits (FromPyObject, ToPyObject, BorrowAsPyObject) for
// owned python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

impl <'p, 's, T : PythonObject<'p>> ToPyObject<'p, 's> for PyPtr<'p, T> {
    type PointerType = &'s PyObject<'p>;
    
    fn to_py_object(&'s self, py: Python<'p>) -> PyResult<'p, &'s PyObject<'p>> {
        Ok(self.as_object())
    }
}

impl <'p, 's, T : PythonObject<'p>> FromPyObject<'p, 's> for PyPtr<'p, T> {
    #[inline]
    fn from_py_object(s : &'s PyObject<'p>) -> PyResult<'p, PyPtr<'p, T>> {
        PyPtr::new(s).downcast_into()
    }
}

// bool


impl <'p, 's> ToPyObject<'p, 's> for bool {
    type PointerType = &'p PyObject<'p>;
    
    #[inline]
    fn to_py_object(&'s self, py: Python<'p>) -> PyResult<'p, &'p PyObject<'p>> {
        Ok(if *self { py.True() } else { py.False() })
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
impl <'p, 's> ToPyObject<'p, 's> for str {
    type PointerType = PyPtr<'p, PyObject<'p>>;
    
    fn to_py_object(&'s self, py : Python<'p>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
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

