// Copyright (c) 2017-present PyO3 Project and Contributors

pub use self::object::PyObject;
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

//#[macro_export]
macro_rules! pyobject_nativetype(
    ($name: ident) => (
        impl $crate::std::convert::AsRef<PyObject> for $name {
            fn as_ref(&self) -> &$crate::PyObject {
                unsafe{$crate::std::mem::transmute(self)}
            }
        }
        impl $crate::PyClone for $name {
            fn clone_ref(&self, _py: $crate::Python) -> Self {
                $name(unsafe{$crate::PyPtr::from_borrowed_ptr(self.as_ptr())})
            }
        }
        impl $crate::python::ToPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }
        impl<'a> $crate::python::ToPyPointer for &'a $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl $crate::python::IntoPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a owned pointer.
            #[inline]
            #[must_use]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                let ptr = self.0.as_ptr();
                $crate::std::mem::forget(self);
                ptr
            }
        }
    );

    ($name: ident, $typeobject: ident, $checkfunction: ident) => {
        pyobject_downcast!($name, $checkfunction);
        pyobject_nativetype!($name, $typeobject);
    };

    ($name: ident, $typeobject: ident) => (
        pyobject_nativetype!($name);

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

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type(_py: Python) {}

            #[inline]
            fn type_object(py: $crate::Python) -> $crate::PyType {
                unsafe { $crate::PyType::from_type_ptr(py, &mut $crate::ffi::$typeobject) }
            }
        }

        impl $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object<'p>(&self, py: $crate::Python<'p>) -> $crate::PyObject {
                $crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut ffi::PyObject) -> R
            {
                f(self.0.as_ptr())
            }
        }

        impl $crate::IntoPyObject for $name
        {
            #[inline]
            fn into_object(self, _py: $crate::Python) -> $crate::PyObject
            {
                unsafe { $crate::std::mem::transmute(self) }
            }
        }

        impl $crate::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::python::PyDowncastInto;

		        let gil = $crate::Python::acquire_gil();
	            let py = gil.python();

                let s = unsafe { $crate::PyString::downcast_from_ptr(
                    py, $crate::ffi::PyObject_Repr(
                        $crate::python::ToPyPointer::as_ptr(self))) };
                let repr_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                let result = f.write_str(&repr_obj.to_string_lossy(py));
                py.release(repr_obj);
                result
            }
        }

        impl $crate::std::fmt::Display for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
		        let gil = $crate::Python::acquire_gil();
	            let py = gil.python();
                use $crate::python::PyDowncastInto;

                let s = unsafe { $crate::PyString::downcast_from_ptr(
                    py, $crate::ffi::PyObject_Str(
                        $crate::python::ToPyPointer::as_ptr(self))) };
                let str_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                let result = f.write_str(&str_obj.to_string_lossy(py));
                py.release(str_obj);
                result
            }
        }
    );
);

macro_rules! pyobject_downcast(
    ($name: ident, $checkfunction: ident) => (
        impl $crate::python::PyDowncastFrom for $name
        {
            fn downcast_from<'a, 'p>(py: $crate::Python<'p>, ob: &'a $crate::PyObject)
                                     -> Result<&'a $name, $crate::PyDowncastError<'p>>
            {
                use $crate::ToPyPointer;

                unsafe {
                    if $crate::ffi::$checkfunction(ob.as_ptr()) > 0 {
                        let ptr = ob as *const _ as *mut u8 as *mut $name;
                        Ok(ptr.as_ref().expect("Failed to call as_ref"))
                    } else {
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }
        }
        impl $crate::python::PyDowncastInto for $name
        {
            fn downcast_into<'p, I>(py: $crate::Python<'p>, ob: I)
                                -> Result<Self, $crate::PyDowncastError<'p>>
                where I: $crate::IntoPyPointer
            {
                unsafe{
                    let ptr = ob.into_ptr();
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($name(PyPtr::from_owned_ptr(ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }

            fn downcast_from_ptr<'p>(py: $crate::Python<'p>, ptr: *mut $crate::ffi::PyObject)
                                     -> Result<$name, $crate::PyDowncastError<'p>>
            {
                unsafe{
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($name(PyPtr::from_owned_ptr(ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }

            fn unchecked_downcast_into<'p, I>(ob: I) -> Self
                where I: $crate::IntoPyPointer
            {
                unsafe{
                    $name(PyPtr::from_owned_ptr(ob.into_ptr()))
                }
            }
        }

        impl<'a> $crate::FromPyObject<'a> for $name
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(py: Python, ob: &'a $crate::PyObject) -> $crate::PyResult<Self>
            {
                unsafe {
                    if ffi::$checkfunction(ob.as_ptr()) != 0 {
                        Ok( $name($crate::pointers::PyPtr::from_borrowed_ptr(ob.as_ptr())) )
                    } else {
                        Err(::PyDowncastError(py, None).into())
                    }
                }
            }
        }

        impl<'a> $crate::FromPyObject<'a> for &'a $name
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(py: Python, ob: &'a $crate::PyObject) -> $crate::PyResult<Self>
            {
                unsafe {
                    if ffi::$checkfunction(ob.as_ptr()) != 0 {
                        Ok($crate::std::mem::transmute(ob))
                    } else {
                        Err($crate::PyDowncastError(py, None).into())
                    }
                }
            }
        }
    );
);

macro_rules! pyobject_convert(
    ($name: ident) => (
        impl $crate::std::convert::From<$name> for $crate::PyObject {
            fn from(ob: $name) -> Self {
                unsafe{$crate::std::mem::transmute(ob)}
            }
        }
    )
);

macro_rules! pyobject_extract(
    ($py:ident, $obj:ident to $t:ty => $body: block) => {
        impl<'source> ::conversion::FromPyObject<'source>
            for $t
        {
            fn extract($py: Python, $obj: &'source ::PyObject) -> $crate::PyResult<Self>
            {
                $body
            }
        }
    }
);


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
mod object;
pub mod exc;

#[cfg(Py_3)]
mod num3;

#[cfg(not(Py_3))]
mod num2;

#[cfg(Py_3)]
mod string;

#[cfg(not(Py_3))]
mod string2;
