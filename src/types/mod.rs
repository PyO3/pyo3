// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

pub use self::any::PyAny;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
pub use self::capsule::PyCapsule;
pub use self::complex::PyComplex;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess, PyTzInfo,
    PyTzInfoAccess,
};
pub use self::dict::{IntoPyDict, PyDict};
pub use self::floatob::PyFloat;
pub use self::function::{PyCFunction, PyFunction};
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::mapping::PyMapping;
pub use self::module::PyModule;
pub use self::num::PyLong;
pub use self::num::PyLong as PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
#[cfg(all(not(Py_LIMITED_API), target_endian = "little"))]
pub use self::string::PyStringData;
pub use self::string::{PyString, PyString as PyUnicode};
pub use self::traceback::PyTraceback;
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;

// Implementations core to all native types
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_base(
    ($name:ty $(;$generics:ident)* ) => {
        unsafe impl<$($generics,)*> $crate::PyNativeType for $name {}

        impl<$($generics,)*> ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                   -> ::std::result::Result<(), ::std::fmt::Error>
            {
                let s = self.repr().or(::std::result::Result::Err(::std::fmt::Error))?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($generics,)*> ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                   -> ::std::result::Result<(), ::std::fmt::Error>
            {
                let s = self.str().or(::std::result::Result::Err(::std::fmt::Error))?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($generics,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python<'_>) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }
    };
);

// Implementations core to all native types except for PyAny (because they don't
// make sense on PyAny / have different implementations).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name:ty $(;$generics:ident)*) => {
        $crate::pyobject_native_type_base!($name $(;$generics)*);

        impl<$($generics,)*> ::std::convert::AsRef<$crate::PyAny> for $name {
            #[inline]
            fn as_ref(&self) -> &$crate::PyAny {
                &self.0
            }
        }

        impl<$($generics,)*> ::std::ops::Deref for $name {
            type Target = $crate::PyAny;

            #[inline]
            fn deref(&self) -> &$crate::PyAny {
                &self.0
            }
        }

        impl<$($generics,)*> $crate::AsPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$($generics,)*> $crate::IntoPy<$crate::Py<$name>> for &'_ $name {
            #[inline]
            fn into_py(self, py: $crate::Python<'_>) -> $crate::Py<$name> {
                use $crate::AsPyPointer;
                unsafe { $crate::Py::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$($generics,)*> ::std::convert::From<&'_ $name> for $crate::Py<$name> {
            #[inline]
            fn from(other: &$name) -> Self {
                use $crate::AsPyPointer;
                use $crate::PyNativeType;
                unsafe { $crate::Py::from_borrowed_ptr(other.py(), other.as_ptr()) }
            }
        }

        impl<'a, $($generics,)*> ::std::convert::From<&'a $name> for &'a $crate::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::PyAny)}
            }
        }
    };
);

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name:ty, $typeobject:expr, $module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        unsafe impl<$($generics,)*> $crate::type_object::PyTypeInfo for $name {
            type AsRefTarget = Self;

            const NAME: &'static str = stringify!($name);
            const MODULE: ::std::option::Option<&'static str> = $module;

            #[inline]
            fn type_object_raw(_py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                // Create a very short lived mutable reference and directly
                // cast it to a pointer: no mutable references can be aliasing
                // because we hold the GIL.
                #[cfg(not(addr_of))]
                unsafe { &mut $typeobject }

                #[cfg(addr_of)]
                unsafe { ::std::ptr::addr_of_mut!($typeobject) }
            }

            $(
                fn is_type_of(ptr: &$crate::PyAny) -> bool {
                    use $crate::AsPyPointer;
                    #[allow(unused_unsafe)]
                    unsafe { $checkfunction(ptr.as_ptr()) > 0 }
                }
            )?
        }
    };
);

// NOTE: This macro is not included in pyobject_native_type_base!
// because rust-numpy has a special implementation.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_extract {
    ($name:ty $(;$generics:ident)*) => {
        impl<'py, $($generics,)*> $crate::FromPyObject<'py> for &'py $name {
            fn extract(obj: &'py $crate::PyAny) -> $crate::PyResult<Self> {
                $crate::PyTryFrom::try_from(obj).map_err(::std::convert::Into::into)
            }
        }
    }
}

/// Declares all of the boilerplate for Python types.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_core {
    ($name:ty, $typeobject:expr, #module=$module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_named!($name $(;$generics)*);
        $crate::pyobject_native_type_info!($name, $typeobject, $module $(, #checkfunction=$checkfunction)? $(;$generics)*);
        $crate::pyobject_native_type_extract!($name $(;$generics)*);
    };
    ($name:ty, $typeobject:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name, $typeobject, #module=::std::option::Option::Some("builtins") $(, #checkfunction=$checkfunction)? $(;$generics)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_sized {
    ($name:ty, $layout:path $(;$generics:ident)*) => {
        unsafe impl $crate::type_object::PyLayout<$name> for $layout {}
        impl $crate::type_object::PySizedLayout<$name> for $layout {}
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::pycell::PyCellBase<$layout>;
            type BaseNativeType = $name;
            type ThreadChecker = $crate::impl_::pyclass::ThreadCheckerStub<$crate::PyObject>;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<Self>;
        }
    }
}

/// Declares all of the boilerplate for Python types which can be inherited from (because the exact
/// Python layout is known).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type {
    ($name:ty, $layout:path, $typeobject:expr $(, #module=$module:expr)? $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name, $typeobject $(, #module=$module)? $(, #checkfunction=$checkfunction)? $(;$generics)*);
        // To prevent inheriting native types with ABI3
        #[cfg(not(Py_LIMITED_API))]
        $crate::pyobject_native_type_sized!($name, $layout $(;$generics)*);
    };
}

mod any;
mod boolobject;
mod bytearray;
mod bytes;
mod capsule;
mod complex;
#[cfg(not(Py_LIMITED_API))]
mod datetime;
mod dict;
mod floatob;
mod function;
mod iterator;
mod list;
mod mapping;
mod module;
mod num;
mod sequence;
mod set;
mod slice;
mod string;
mod traceback;
mod tuple;
mod typeobject;
