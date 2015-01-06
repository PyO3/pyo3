pub use self::object::PyObject;
pub use self::typeobject::PyType;
pub use self::module::PyModule;
pub use self::string::{PyBytes, PyUnicode};
pub use self::iterator::PyIterator;
use python::{Python, PythonObject};
use pyptr::{PyPtr, PythonPointer};
use err::{PyErr, PyResult};
use ffi::{Py_ssize_t};

macro_rules! pythonobject_newtype_only_pythonobject(
    ($name: ident) => (
        pub struct $name<'p>(::objects::PyObject<'p>);
        
        impl <'p> ::python::PythonObject<'p> for $name<'p> {
            #[inline]
            fn as_object<'a>(&'a self) -> &'a ::objects::PyObject<'p> {
                &self.0
            }
            
            #[inline]
            unsafe fn unchecked_downcast_from<'a>(obj: &'a ::objects::PyObject<'p>) -> &'a $name<'p> {
                ::std::mem::transmute(obj)
            }
        }
    )
);

macro_rules! pyobject_newtype(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (
        pythonobject_newtype_only_pythonobject!($name);
        
        impl <'p> ::python::PythonObjectWithCheckedDowncast<'p> for $name<'p> {
            #[inline]
            fn downcast_from<'a>(obj : &'a ::objects::PyObject<'p>) -> Option<&'a $name<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(::python::PythonObject::as_ptr(obj)) {
                        Some(::python::PythonObject::unchecked_downcast_from(obj))
                    } else {
                        None
                    }
                }
            }
        }

        impl <'p> ::python::PythonObjectWithTypeObject<'p> for $name<'p> {
            #[inline]
            fn type_object(py: ::python::Python<'p>, _ : Option<&Self>) -> &'p ::objects::PyType<'p> {
                unsafe { ::objects::PyType::from_type_ptr(py, &mut ::ffi::$typeobject) }
            }
        }
    )
);

mod object;
mod typeobject;
mod module;
mod string;
mod dict;
mod iterator;

pyobject_newtype!(PyList, PyList_Check, PyList_Type);


pyobject_newtype!(PyBool, PyBool_Check, PyBool_Type);

impl <'p> PyBool<'p> {
    #[inline]
    pub fn get(py: Python<'p>, val: bool) -> &'p PyBool<'p> {
        if val { py.True() } else { py.False() }
    }
    
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { ::ffi::Py_True() }
    }
}

pyobject_newtype!(PyTuple, PyTuple_Check, PyTuple_Type);

impl <'p> PyTuple<'p> {
    pub fn new(py: Python<'p>, elements: &[&PyObject<'p>]) -> PyResult<'p, PyPtr<'p, PyTuple<'p>>> {
        unsafe {
            let len = elements.len();
            let ptr = ::ffi::PyTuple_New(len as Py_ssize_t);
            let t = try!(::err::result_from_owned_ptr(py, ptr)).unchecked_downcast_into::<PyTuple>();
            for (i, e) in elements.iter().enumerate() {
                ::ffi::PyTuple_SET_ITEM(ptr, i as Py_ssize_t, e.steal_ptr());
            }
            Ok(t)
        }
    }
    
    #[inline]
    pub fn len(&self) -> uint {
        // non-negative Py_ssize_t should always fit into Rust uint
        unsafe {
            ::ffi::PyTuple_GET_SIZE(self.as_ptr()) as uint
        }
    }
    
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [&'a PyObject<'p>] {
        // This is safe because &PyObject has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        unsafe {
            let ptr = self.as_ptr() as *mut ::ffi::PyTupleObject;
            ::std::mem::transmute(::std::raw::Slice {
                data: (*ptr).ob_item.as_ptr(),
                len: self.len()
            })
        }
    }
}

impl<'p> ::std::ops::Index<uint> for PyTuple<'p> {
    type Output = PyObject<'p>;

    #[inline]
    fn index<'a>(&'a self, index: &uint) -> &'a PyObject<'p> {
        // use as_slice() to use the normal Rust bounds checking when indexing
        self.as_slice()[*index]
    }
}

