// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

pub use self::object::PyObject;
pub use self::typeobject::PyType;
pub use self::module::PyModule;
pub use self::string::{PyBytes, PyString, PyStringData};
pub use self::iterator::PyIterator;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::tuple::{PyTuple, NoArgs};
pub use self::dict::PyDict;
pub use self::list::PyList;
pub use self::num::{PyLong, PyFloat};
pub use self::sequence::PySequence;
pub use self::slice::PySlice;
pub use self::set::{PySet, PyFrozenSet};

#[macro_export]
macro_rules! pyobject_newtype(
    ($name: ident) => (
        py_impl_to_py_object_for_python_object!($name);
        py_impl_from_py_object_for_python_object!($name);

        impl $crate::PythonObject for $name {
            #[inline]
            fn as_object(&self) -> &$crate::PyObject {
                &self.0
            }

            #[inline]
            fn into_object(self) -> $crate::PyObject {
                self.0
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: $crate::PyObject) -> Self {
                $name(obj)
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a $crate::PyObject) -> &'a Self {
                ::std::mem::transmute(obj)
            }
        }
    );
    ($name: ident, $checkfunction: ident) => (
        pyobject_newtype!($name);

        impl ::python::PythonObjectWithCheckedDowncast for $name {
            #[inline]
            fn downcast_from<'p>(py: ::python::Python<'p>, obj: ::objects::object::PyObject) -> Result<$name, ::python::PythonObjectDowncastError<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(obj.as_ptr()) != 0 {
                        Ok($name(obj))
                    } else {
                        Err(::python::PythonObjectDowncastError(py, None))
                    }
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: ::python::Python<'p>, obj: &'a ::objects::object::PyObject) -> Result<&'a $name, ::python::PythonObjectDowncastError<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(obj.as_ptr()) != 0 {
                        Ok(::std::mem::transmute(obj))
                    } else {
                        Err(::python::PythonObjectDowncastError(py, None))
                    }
                }
            }
        }
    );
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (
        pyobject_newtype!($name, $checkfunction);

        impl $crate::class::typeob::PyTypeInfo for $name {
            type Type = ();

            #[inline]
            fn size() -> usize {
                $crate::std::mem::size_of::<$name>()
            }

            #[inline]
            fn offset() -> isize {
                0
            }

            #[inline]
            fn type_name() -> &'static str {
                stringify!($name)
            }
            #[inline]
            fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                unsafe { &mut $crate::ffi::$typeobject }
            }
        }
    );
);

macro_rules! extract(
    ($obj:ident to $t:ty; $py:ident => $body: block) => {
        impl <'source> ::conversion::FromPyObject<'source>
            for $t
        {
            fn extract($py: Python, $obj: &'source PyObject) -> PyResult<Self> {
                $body
            }
        }
    }
);

#[macro_export]
macro_rules! pyobject_newtype_(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (
        impl $crate::python::AsPy for $name {
            #[inline]
            fn py(&self) -> Python {
                unsafe { $crate::python::Python::assume_gil_acquired() }
            }
        }

        impl $crate::python::ToPythonPointer for $name {
            #[inline]
            fn as_ptr(&self) -> *mut ffi::PyObject {
                self as *const _ as *mut ffi::PyObject
            }

            #[inline]
            #[must_use]
            fn steal_ptr(self, _py: Python) -> *mut ffi::PyObject {
                let ptr = self.as_ptr();
                $crate::mem::forget(self);
                ptr
            }
        }

        impl $crate::class::typeob::PyTypeInfo for $name {
            type Type = ();

            #[inline]
            fn size() -> usize {
                $crate::std::mem::size_of::<$name>()
            }

            #[inline]
            fn offset() -> isize {
                0
            }

            #[inline]
            fn type_name() -> &'static str {
                stringify!($name)
            }
            #[inline]
            fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                unsafe { &mut $crate::ffi::$typeobject }
            }
        }
    );
);



mod object;
mod typeobject;
mod module;
mod string;
mod dict;
mod iterator;
mod boolobject;
mod bytearray;
mod tuple;
mod list;
mod num;
mod sequence;
mod slice;
mod set;
pub mod exc;

use std;

use ffi;
use class::typeob::PyTypeInfo;

pub struct PyObj;


impl PyTypeInfo for PyObj {
    type Type = ();

    #[inline]
    fn size() -> usize {
        std::mem::size_of::<ffi::PyObject>()
    }

    #[inline]
    fn offset() -> isize {
        0
    }

    #[inline]
    fn type_name() -> &'static str {
        "PyObject"
    }

    #[inline]
    fn type_object() -> &'static mut ffi::PyTypeObject {
        unsafe { &mut ffi::PyBaseObject_Type }
    }
}

impl PyTypeInfo for PyObject {
    type Type = ();

    #[inline]
    fn size() -> usize {
        std::mem::size_of::<ffi::PyObject>()
    }

    #[inline]
    fn offset() -> isize {
        0
    }

    #[inline]
    fn type_name() -> &'static str {
        "PyObject"
    }

    #[inline]
    fn type_object() -> &'static mut ffi::PyTypeObject {
        unsafe { &mut ffi::PyBaseObject_Type }
    }
}
