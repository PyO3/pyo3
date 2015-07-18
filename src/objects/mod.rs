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

pub use self::string::{PyBytes, PyUnicode};
// Directly exporting self::string::PyString causes a compiler error?
#[cfg(feature="python27-sys")]
pub use self::string::PyBytes as PyString;
#[cfg(feature="python3-sys")]
pub use self::string::PyUnicode as PyString;

#[cfg(feature="python27-sys")]
pub use self::iterator::PyIterator;
pub use self::boolobject::PyBool;
pub use self::tuple::{PyTuple, NoArgs};
pub use self::dict::PyDict;
pub use self::list::PyList;
#[cfg(feature="python27-sys")]
pub use self::num::PyInt;
#[cfg(feature="python3-sys")]
pub use self::num::PyLong as PyInt;
pub use self::num::{PyLong, PyFloat};

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
macro_rules! pyobject_to_pyobject(
    ($name: ident) => (
        impl <'p, 's> ::conversion::ToPyObject<'p> for $name<'s> {
            type ObjectType = $name<'p>;

            #[inline]
            fn to_py_object(&self, py: Python<'p>) -> $name<'p> {
                self.clone().into_py_object(py)
            }

            #[inline]
            fn into_py_object(self, _py: Python<'p>) -> $name<'p> {
                // Transmute the lifetime.
                // This is safe, because both lifetime variables represent the same lifetime:
                // that of the python GIL acquisition.
                unsafe { ::std::mem::transmute(self) }
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: Python<'p>, f: F) -> R
              where F: FnOnce(*mut ffi::PyObject) -> R {
                f(self.as_ptr())
            }
        }
    )
);

macro_rules! pyobject_newtype(
    ($name: ident) => (
        /// Clone returns another reference to the Python object,
        /// thus incrementing the reference count by 1.
        impl <'p> Clone for $name<'p> {
            #[inline]
            fn clone(&self) -> Self {
                $name(self.0.clone())
            }
        }

        pyobject_to_pyobject!($name);

        impl <'p> ::python::PythonObject<'p> for $name<'p> {
            #[inline]
            fn as_object(&self) -> &::objects::object::PyObject<'p> {
                &self.0
            }
            
            #[inline]
            fn into_object(self) -> ::objects::object::PyObject<'p> {
                self.0
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: ::objects::object::PyObject<'p>) -> Self {
                $name(obj)
            }
            
            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a ::objects::object::PyObject<'p>) -> &'a Self {
                ::std::mem::transmute(obj)
            }

            #[inline]
            fn python(&self) -> Python<'p> {
                self.0.python()
            }
        }
    );
    ($name: ident, $checkfunction: ident) => (
        pyobject_newtype!($name);
        
        impl <'p> ::python::PythonObjectWithCheckedDowncast<'p> for $name<'p> {
            #[inline]
            fn downcast_from(obj : ::objects::object::PyObject<'p>) -> Result<$name<'p>, ::python::PythonObjectDowncastError<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(::python::ToPythonPointer::as_ptr(&obj)) != 0 {
                        Ok($name(obj))
                    } else {
                        Err(::python::PythonObjectDowncastError(::python::PythonObject::python(&obj)))
                    }
                }
            }
            
            #[inline]
            fn downcast_borrow_from<'a>(obj : &'a ::objects::object::PyObject<'p>) -> Result<&'a $name<'p>, ::python::PythonObjectDowncastError<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(::python::ToPythonPointer::as_ptr(obj)) != 0 {
                        Ok(::std::mem::transmute(obj))
                    } else {
                        Err(::python::PythonObjectDowncastError(::python::PythonObject::python(obj)))
                    }
                }
            }
        }
    );
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (
        pyobject_newtype!($name, $checkfunction);
        
        impl <'p> ::python::PythonObjectWithTypeObject<'p> for $name<'p> {
            #[inline]
            fn type_object(py: ::python::Python<'p>) -> ::objects::typeobject::PyType<'p> {
                unsafe { ::objects::typeobject::PyType::from_type_ptr(py, &mut ::ffi::$typeobject) }
            }
        }
    );
);

mod object;
mod typeobject;
mod module;
mod string;
mod dict;
#[cfg(feature="python27-sys")]
mod iterator;
mod boolobject;
mod tuple;
mod list;
mod num;
pub mod exc;

#[cfg(feature="python27-sys")]
pub mod oldstyle;

