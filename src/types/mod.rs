// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the python interpreter such as `int`, `str` and `tuple`

pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::complex::PyComplex;
pub use self::datetime::PyDeltaAccess;
pub use self::datetime::{PyDate, PyDateTime, PyDelta, PyTime, PyTzInfo};
pub use self::datetime::{PyDateAccess, PyTimeAccess};
pub use self::dict::IntoPyDict;
pub use self::dict::PyDict;
pub use self::floatob::PyFloat;
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::module::PyModule;
#[cfg(not(Py_3))]
pub use self::num2::{PyInt, PyLong};
#[cfg(Py_3)]
pub use self::num3::PyLong;
#[cfg(Py_3)]
pub use self::num3::PyLong as PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
#[cfg(Py_3)]
pub use self::string::{PyBytes, PyString, PyString as PyUnicode};
#[cfg(not(Py_3))]
pub use self::string2::{PyBytes, PyString, PyUnicode};
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;
use ffi;
use python::ToPyPointer;

/// Implements a typesafe conversions throught [FromPyObject], given a typecheck function as second
/// parameter
#[macro_export]
macro_rules! pyobject_downcast (
    ($name: ty, $checkfunction: path $(,$type_param: ident)*) => (
        impl<'a, $($type_param,)*> $crate::FromPyObject<'a> for &'a $name
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(ob: &'a $crate::types::PyObjectRef) -> $crate::PyResult<Self>
            {
                unsafe {
                    if $checkfunction(ob.as_ptr()) != 0 {
                        Ok(&*(ob as *const $crate::types::PyObjectRef as *const $name))
                    } else {
                        Err($crate::PyDowncastError.into())
                    }
                }
            }
        }
    );
);

#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name: ty $(,$type_param: ident)*) => {
        impl<$($type_param,)*> $crate::PyNativeType for $name {}

        impl<$($type_param,)*> ::std::convert::AsRef<$crate::types::PyObjectRef> for $name {
            fn as_ref(&self) -> &$crate::types::PyObjectRef {
                unsafe{&*(self as *const $name as *const $crate::types::PyObjectRef)}
            }
        }

        impl<$($type_param,)*> $crate::PyObjectWithToken for $name {
            #[inline]
            fn py(&self) -> $crate::Python {
                unsafe { $crate::Python::assume_gil_acquired() }
            }
        }

        impl<$($type_param,)*> $crate::python::ToPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$($type_param,)*> PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                self.as_ptr() == o.as_ptr()
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type (
    ($name: ty, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_type_convert!($name, $typeobject, $checkfunction $(,$type_param)*);

        impl<'a, $($type_param,)*> ::std::convert::From<&'a $name> for &'a $crate::types::PyObjectRef {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::types::PyObjectRef)}
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type_convert(
    ($name: ty, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl<$($type_param,)*> $crate::typeob::PyTypeInfo for $name {
            type Type = ();
            type BaseType = $crate::types::PyObjectRef;

            const NAME: &'static str = stringify!($name);
            const SIZE: usize = ::std::mem::size_of::<$crate::ffi::PyObject>();
            const OFFSET: isize = 0;

            #[inline]
            unsafe fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                &mut $typeobject
            }

            fn is_instance(ptr: &$crate::types::PyObjectRef) -> bool {
                #[allow(unused_unsafe)]
                unsafe { $checkfunction(ptr.as_ptr()) > 0 }
            }
        }

        impl<$($type_param,)*> $crate::typeob::PyTypeObject for $name {
            #[inline]
            fn init_type() {}

            #[inline]
            fn type_object() -> $crate::Py<$crate::types::PyType> {
                $crate::types::PyType::new::<$name>()
            }
        }

        impl<$($type_param,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())}
            }
        }

        impl<$($type_param,)*> ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.repr().map_err(|_| ::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($type_param,)*> ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.str().map_err(|_| ::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }
    };
);

/// Represents general python instance.
#[repr(transparent)]
pub struct PyObjectRef(::PyObject);
pyobject_native_type_named!(PyObjectRef);
pyobject_native_type_convert!(PyObjectRef, ffi::PyBaseObject_Type, ffi::PyObject_Check);

mod boolobject;
mod bytearray;
mod complex;
mod datetime;
mod dict;
pub mod exceptions;
mod floatob;
mod iterator;
mod list;
mod module;
mod sequence;
mod set;
mod slice;
mod stringutils;
mod tuple;
mod typeobject;

#[macro_use]
mod num_common;

#[cfg(Py_3)]
mod num3;

#[cfg(not(Py_3))]
mod num2;

#[cfg(Py_3)]
mod string;

#[cfg(not(Py_3))]
mod string2;
