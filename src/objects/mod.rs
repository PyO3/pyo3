// Copyright (c) 2017-present PyO3 Project and Contributors

pub use self::typeobject::PyType;
pub use self::module::PyModule;
pub use self::iterator::PyIterator;
pub use self::boolobject::PyBool;
pub use self::bytearray::{PyByteArray};
pub use self::tuple::{PyTuple, NoArgs};
pub use self::dict::PyDict;
pub use self::list::PyList;
pub use self::floatob::PyFloat;
pub use self::sequence::PySequence;
pub use self::slice::{PySlice, PySliceIndices};
pub use self::set::{PySet, PyFrozenSet};
pub use self::stringdata::PyStringData;

#[cfg(Py_3)]
pub use self::string::{PyBytes, PyString};

#[cfg(not(Py_3))]
pub use self::string2::{PyBytes, PyString};

#[cfg(Py_3)]
pub use self::num3::PyLong;
#[cfg(Py_3)]
pub use self::num3::PyLong as PyInt;

#[cfg(not(Py_3))]
pub use self::num2::{PyInt, PyLong};


macro_rules! pyobject_downcast(
    ($name: ident, $checkfunction: ident) => (
        impl $crate::python::PyDowncastFrom for $name
        {
            fn downcast_from(ob: &$crate::PyObjectRef)
                             -> Result<&$name, $crate::PyDowncastError>
            {
                use $crate::{ToPyPointer, PyObjectWithToken};

                unsafe {
                    if $crate::ffi::$checkfunction(ob.as_ptr()) > 0 {
                        Ok($crate::std::mem::transmute(ob))
                    } else {
                        Err($crate::PyDowncastError(ob.token(), None))
                    }
                }
            }

            unsafe fn unchecked_downcast_from(ob: &$crate::PyObjectRef) -> &Self
            {
                $crate::std::mem::transmute(ob)
            }
            unsafe fn unchecked_mut_downcast_from(ob: &$crate::PyObjectRef) -> &mut Self
            {
                #[allow(mutable_transmutes)]
                $crate::std::mem::transmute(ob)
            }
        }

        impl<'a> $crate::FromPyObject<'a> for &'a $name
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(ob: &'a $crate::PyObjectRef) -> $crate::PyResult<Self>
            {
                use instance::PyObjectWithToken;
                unsafe {
                    if $crate::ffi::$checkfunction(ob.as_ptr()) != 0 {
                        Ok($crate::std::mem::transmute(ob))
                    } else {
                        Err($crate::PyDowncastError(ob.token(), None).into())
                    }
                }
            }
        }
    );
);

macro_rules! pyobject_convert(
    ($name: ident) => (
        impl<'a> $crate::std::convert::From<&'a $name> for &'a $crate::PyObjectRef {
            fn from(ob: &'a $name) -> Self {
                unsafe{$crate::std::mem::transmute(ob)}
            }
        }
    )
);

macro_rules! pyobject_nativetype(
    ($name: ident) => {
        impl $crate::PyNativeType for $name {}

        impl $crate::std::convert::AsRef<$crate::PyObjectRef> for $name {
            fn as_ref(&self) -> &$crate::PyObjectRef {
                unsafe{$crate::std::mem::transmute(self)}
            }
        }
        impl $crate::PyObjectWithToken for $name {
            #[inline]
            fn token<'p>(&'p self) -> $crate::Python<'p> {
                unsafe { $crate::Python::assume_gil_acquired() }
            }
        }
        impl $crate::python::ToPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }
        impl $crate::python::IntoPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                let ptr = self.0.as_ptr();
                unsafe { $crate::ffi::Py_INCREF(ptr); }
                ptr
            }
        }
        impl<'a> $crate::python::IntoPyPointer for &'a $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                let ptr = self.0.as_ptr();
                unsafe { $crate::ffi::Py_INCREF(ptr); }
                ptr
            }
        }
        impl PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                self.as_ptr() == o.as_ptr()
            }
        }
    };

    ($name: ident, $typeobject: ident, $checkfunction: ident) => {
        pyobject_nativetype!($name);

        impl $crate::typeob::PyTypeInfo for $name {
            type Type = ();
            const NAME: &'static str = stringify!($name);

            #[inline]
            fn size() -> usize {
                $crate::std::mem::size_of::<$crate::ffi::PyObject>()
            }
            #[inline]
            fn offset() -> isize {
                0
            }
            #[inline]
            unsafe fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                &mut $crate::ffi::$typeobject
            }
            #[inline]
            fn is_instance(ptr: *mut $crate::ffi::PyObject) -> bool {
                #[allow(unused_unsafe)]
                unsafe { $crate::ffi::$checkfunction(ptr) > 0 }
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type(_py: $crate::Python) {}

            #[inline]
            fn type_object<'p>(py: $crate::Python<'p>) -> &'p $crate::PyType {
                unsafe { $crate::PyType::from_type_ptr(py, &mut $crate::ffi::$typeobject) }
            }
        }

        impl $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object<'p>(&self, py: $crate::Python<'p>) -> $crate::PyObject {
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())}
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut $crate::ffi::PyObject) -> R
            {
                f(self.0.as_ptr())
            }
        }

        impl<'a> $crate::IntoPyObject for &'a $name
        {
            #[inline]
            fn into_object<'p>(self, py: $crate::Python) -> $crate::PyObject {
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl $crate::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.repr().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }

        impl $crate::std::fmt::Display for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = try!(self.str().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&s.to_string_lossy())
            }
        }

        pyobject_downcast!($name, $checkfunction);
};
);

macro_rules! pyobject_extract(
    ($py:ident, $obj:ident to $t:ty => $body: block) => {
        impl<'source> $crate::FromPyObject<'source> for $t
        {
            fn extract($obj: &'source $crate::PyObjectRef) -> $crate::PyResult<Self>
            {
                #[allow(unused_imports)]
                use objectprotocol::ObjectProtocol;

                $body
            }
        }
    }
);


use python::ToPyPointer;

/// Represents general python instance.
pub struct PyObjectRef(::PyObject);
pyobject_nativetype!(PyObjectRef, PyBaseObject_Type, PyObject_Check);

mod typeobject;
mod module;
mod dict;
mod iterator;
mod boolobject;
mod bytearray;
mod tuple;
mod list;
mod floatob;
mod sequence;
mod slice;
mod stringdata;
mod stringutils;
mod set;
pub mod exc;

#[cfg(Py_3)]
mod num3;

#[cfg(not(Py_3))]
mod num2;

#[cfg(Py_3)]
mod string;

#[cfg(not(Py_3))]
mod string2;
