use object::{PythonObject, PyObject};
use python::Python;
use ffi;
use libc::c_char;
use std;

pub struct PyTypeObject<'p> {
    cell : std::cell::UnsafeCell<ffi::PyTypeObject>,
    py : Python<'p>
}

impl <'p> PythonObject<'p> for PyTypeObject<'p> {
    #[inline]
    fn from_object<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyTypeObject<'p>> {
        unsafe {
            if ffi::PyType_Check(obj.as_ptr()) {
                Some(std::mem::transmute(obj))
            } else {
                None
            }
        }
    }
    
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn python(&self) -> Python<'p> {
        self.py
    }
    
    fn type_object(_ : Option<&Self>) -> &'p PyTypeObject<'p> {
        panic!()
    }
}
/*
impl PyTypeObject {
	pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObjectRaw {
		// safe because the PyObject is only accessed while holding the GIL
		(unsafe { self.cell.get() })
	}

	pub unsafe fn from_type_ptr(_ : &Python, p : *mut ffi::PyTypeObjectRaw) -> &PyTypeObject {
		debug_assert!(p.is_not_null())
		&*(p as *mut PyTypeObject)
	}

	/// Return true if self is a subtype of b.
	pub fn is_subtype_of(&self, b : &PyTypeObject) -> bool {
		unsafe { ffi::PyType_IsSubtype(self.as_type_ptr(), b.as_type_ptr()) != 0 }
	}

	/// Return true if obj is an instance of self.
	pub fn is_instance(&self, obj : &PyObject) -> bool {
		obj.get_type().is_subtype_of(self)
	}
}
*/

