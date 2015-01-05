use std;
use std::ops::Deref;
use ffi;
use object::{PythonObject, PyObject};
use err::{PyResult, PyErr};
use python::Python;
//use conversion::{FromPyObject, ToPyObject};
//use PyResult;


/// Owned pointer to python object.
/// The PyPtr<T> owns one reference to a python object.
/// Python objects are reference-counted, so it is possible to have
/// multiple PyPtr<T> objects pointing to the same object, like Rc<T>.
pub struct PyPtr<'p, T : 'p + PythonObject<'p>>(&'p T);

// impl Deref for PyPtr
impl <'p, T : 'p + PythonObject<'p>> Deref for PyPtr<'p, T> {
    type Target = T;
    
    #[inline]
    fn deref(&self) -> &T {
        debug_assert!(self.0.as_object().get_refcnt() > 0);
        self.0
    }
}

// impl Drop for PyPtr
#[unsafe_destructor]
impl<'p, T : 'p + PythonObject<'p>> Drop for PyPtr<'p, T> {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.0.as_object().get_refcnt() > 0);
        unsafe { ffi::Py_DECREF(self.as_ptr()) }
    }
}

// impl Clone for PyPtr
impl<'p, T : 'p + PythonObject<'p>> Clone for PyPtr<'p, T> {
    #[inline]
    fn clone(&self) -> PyPtr<'p, T> {
        unsafe { ffi::Py_INCREF(self.as_ptr()) };
        PyPtr(self.0)
    }
}

// impl Show for PyPtr
impl<'p, T : 'p + PythonObject<'p> + std::fmt::Show> std::fmt::Show for PyPtr<'p, T> {
    #[inline]
    fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.deref().fmt(f)
    }
}

// impl PyPtr
impl<'p, T : 'p + PythonObject<'p>> PyPtr<'p, T> {
    /// Creates a new PyPtr instance from a borrowed reference.
    /// This increments the reference count.
    #[inline]
    pub fn new(obj : &T) -> PyPtr<'p, T> {
        debug_assert!(obj.as_object().get_refcnt() > 0);
        let obj_extended_life : &T = unsafe {
            ffi::Py_INCREF(obj.as_ptr());
            // transmuting from &T to &'p T is safe because we just incremented the reference count,
            // and the &'p T is used only within the PyPtr -- the reference returned by Deref has
            // the lifetime restricted to the PyPtr's lifetime.
            std::mem::transmute(obj)
        };
        PyPtr(obj_extended_life)
    }
}

/// The PythonPointer trait allows extracting an FFI pointer.
pub trait PythonPointer {
    /// Gets the FFI pointer (borrowed reference).
    fn as_ptr(&self) -> *mut ffi::PyObject;
    /// Gets the FFI pointer (owned reference).
    /// If the implementation of this trait is an owned pointer, this steals the reference.
    fn steal_ptr(self) -> *mut ffi::PyObject;
}

impl <'p, T : 'p + PythonObject<'p>> PythonPointer for PyPtr<'p, T> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.deref().as_ptr()
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        // Destruct the PyPtr without decrementing the reference count
        let p = self.deref().as_ptr();
        unsafe { std::mem::forget(self) };
        p
    }
}

// &PyObject (etc.) is also a PythonPointer
// (but steal_ptr increases the reference count)
impl <'p, 'a, T : 'p + PythonObject<'p>> PythonPointer for &'a T {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.deref().as_ptr()
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        PyPtr::new(self).steal_ptr()
    }
}

// Option<PythonPointer> can be used to extract a nullable FFI pointer.
impl <T : PythonPointer> PythonPointer for Option<T> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref p) => p.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        match self {
            Some(p) => p.steal_ptr(),
            None => std::ptr::null_mut()
        }
    }
}

// impl PyPtr<PyObject>
impl<'p> PyPtr<'p, PyObject<'p>> {
    #[inline]
    pub unsafe fn from_owned_ptr(py : Python<'p>, p : *mut ffi::PyObject) -> PyPtr<'p, PyObject<'p>> {
        debug_assert!(!p.is_null() && ffi::Py_REFCNT(p) > 0);
        PyPtr(PyObject::from_ptr(py, p))
    }


    #[inline]
    pub unsafe fn from_borrowed_ptr(py : Python<'p>, p : *mut ffi::PyObject) -> PyPtr<'p, PyObject<'p>> {
        debug_assert!(!p.is_null() && ffi::Py_REFCNT(p) > 0);
        ffi::Py_INCREF(p);
        PyPtr(PyObject::from_ptr(py, p))
    }

    #[inline]
    pub unsafe fn from_owned_ptr_opt(py : Python<'p>, p : *mut ffi::PyObject) -> Option<PyPtr<'p, PyObject<'p>>> {
        if p.is_null() { None } else { Some(PyPtr::from_owned_ptr(py, p)) }
    }

    #[inline]
    pub unsafe fn from_borrowed_ptr_opt(py : Python<'p>, p : *mut ffi::PyObject) -> Option<PyPtr<'p, PyObject<'p>>> {
        if p.is_null() { None } else { Some(PyPtr::from_borrowed_ptr(py, p)) }
    }
    
    /// Casts the PyPtr<PyObject> to a PyPtr of a concrete python object type.
    /// Returns a python TypeError if the object is not of the expected type.
    #[inline]
    pub fn downcast_into<T : PythonObject<'p>>(self) -> PyResult<'p, PyPtr<'p, T>> {
        // TODO: avoid unnecessary IncRef/DefRef
        self.deref().downcast().map(PyPtr::new)
    }
}

