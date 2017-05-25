// Copyright (c) 2017-present PyO3 Project and Contributors

pub use self::object::PyObject;
pub use self::typeobject::PyType;
pub use self::module::PyModule;
pub use self::string::{PyBytes, PyString, PyStringData};
//pub use self::iterator::PyIterator;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::tuple::{PyTuple, NoArgs};
pub use self::dict::PyDict;
pub use self::list::PyList;
pub use self::num::{PyLong, PyFloat};
//pub use self::sequence::PySequence;
pub use self::slice::PySlice;
pub use self::set::{PySet, PyFrozenSet};


#[macro_export]
macro_rules! pyobject_newtype(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (

        impl<'p> $crate::python::AsPy<'p> for &'p $name {
            #[inline]
            fn py<'a>(&'a self) -> Python<'p> {
                unsafe { $crate::python::Python::assume_gil_acquired() }
            }
        }

        impl $crate::python::ToPythonPointer for $name {
            #[inline]
            fn as_ptr(&self) -> *mut ffi::PyObject {
                self as *const _ as *mut ffi::PyObject
            }
        }

        impl<'a> $crate::python::ToPythonPointer for &'a $name {
            #[inline]
            fn as_ptr(&self) -> *mut ffi::PyObject {
                self as *const _ as *mut ffi::PyObject
            }
        }

        impl $crate::typeob::PyTypeInfo for $name {
            type Type = ();

            #[inline]
            fn size() -> usize {
                $crate::std::mem::size_of::<ffi::PyObject>()
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

macro_rules! pyobject_extract(
    ($obj:ident to $t:ty => $body: block) => {
        impl<'source> ::conversion::FromPyObject<'source>
            for $t
        {
            fn extract<S>($obj: &'source ::Py<'source, S>) -> $crate::PyResult<Self>
                where S: ::typeob::PyTypeInfo
            {
                $body
            }
        }
    }
);


mod typeobject;
mod module;
mod string;
mod dict;
//mod iterator;
mod boolobject;
mod bytearray;
mod tuple;
mod list;
mod num;
//mod sequence;
mod slice;
mod set;
mod object;
pub mod exc;
