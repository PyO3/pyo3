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
//pub use self::list::PyList;
pub use self::num::{PyLong, PyFloat};
//pub use self::sequence::PySequence;
pub use self::slice::PySlice;
//pub use self::set::{PySet, PyFrozenSet};


#[macro_export]
macro_rules! pyobject_newtype(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (

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

        impl $crate::python::PythonObjectWithToken for $name {
            fn token<'p>(&'p self) -> $crate::python::Python<'p> {
                self.0.token()
            }
        }

        impl $crate::std::fmt::Debug for $name {
            default fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                           -> Result<(), $crate::std::fmt::Error>
            {
                let py = <$name as $crate::python::PythonObjectWithToken>::token(self);
                let s = unsafe { $crate::Py::<$crate::PyString>::cast_from_owned_nullptr(
                    py, $crate::ffi::PyObject_Repr(
                        $crate::python::ToPythonPointer::as_ptr(self))) };
                let repr_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                f.write_str(&repr_obj.to_string_lossy())
            }
        }

        impl $crate::std::fmt::Display for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                let py = <$name as $crate::python::PythonObjectWithToken>::token(self);
                let s = unsafe { $crate::Py::<$crate::PyString>::cast_from_owned_nullptr(
                    py, $crate::ffi::PyObject_Str(
                        $crate::python::ToPythonPointer::as_ptr(self))) };
                let str_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                f.write_str(&str_obj.to_string_lossy())
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
//mod list;
mod num;
//mod sequence;
mod slice;
// mod set;
mod object;
pub mod exc;
