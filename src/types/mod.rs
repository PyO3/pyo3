// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

pub use self::any::PyAny;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
pub use self::complex::PyComplex;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess, PyTzInfo,
};
pub use self::dict::{IntoPyDict, PyDict};
pub use self::floatob::PyFloat;
pub use self::function::{PyCFunction, PyFunction};
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::module::PyModule;
pub use self::num::PyLong;
pub use self::num::PyLong as PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
pub(crate) use self::string::with_tmp_string;
pub use self::string::{PyString, PyString as PyUnicode};
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;

// Implementations core to all native types
#[macro_export]
macro_rules! pyobject_native_type_base(
    ($name: ty $(;$generics: ident)* ) => {
        unsafe impl<$($generics,)*> $crate::PyNativeType for $name {}

        impl<$($generics,)*> std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter)
                   -> std::result::Result<(), std::fmt::Error>
            {
                let s = self.repr().map_err(|_| std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($generics,)*> std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter)
                   -> std::result::Result<(), std::fmt::Error>
            {
                let s = self.str().map_err(|_| std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($generics,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$($generics,)*> PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                use $crate::AsPyPointer;

                self.as_ptr() == o.as_ptr()
            }
        }
    };
);

// Implementations core to all native types except for PyAny (because they don't
// make sense on PyAny / have different implementations).
#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name: ty $(;$generics: ident)*) => {
        $crate::pyobject_native_type_base!($name $(;$generics)*);

        impl<$($generics,)*> std::convert::AsRef<$crate::PyAny> for $name {
            #[inline]
            fn as_ref(&self) -> &$crate::PyAny {
                &self.0
            }
        }

        impl<$($generics,)*> std::ops::Deref for $name {
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
            fn into_py(self, py: $crate::Python) -> $crate::Py<$name> {
                use $crate::AsPyPointer;
                unsafe { $crate::Py::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$($generics,)*> From<&'_ $name> for $crate::Py<$name> {
            #[inline]
            fn from(other: &$name) -> Self {
                use $crate::AsPyPointer;
                use $crate::PyNativeType;
                unsafe { $crate::Py::from_borrowed_ptr(other.py(), other.as_ptr()) }
            }
        }

        impl<'a, $($generics,)*> std::convert::From<&'a $name> for &'a $crate::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::PyAny)}
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type_core {
    ($name: ty, $layout: path, $typeobject: expr, $module: expr $(, $checkfunction:path)? $(;$generics: ident)*) => {
        unsafe impl $crate::type_object::PyLayout<$name> for $layout {}
        $crate::pyobject_native_type_named!($name $(;$generics)*);
        $crate::pyobject_native_type_info!($name, $layout, $typeobject, $module $(, $checkfunction)? $(;$generics)*);
        $crate::pyobject_native_type_extract!($name $(;$generics)*);
    }
}

#[macro_export]
macro_rules! pyobject_native_type_sized {
    ($name: ty, $layout: path $(;$generics: ident)*) => {
        // To prevent inheriting native types with ABI3
        #[cfg(not(Py_LIMITED_API))]
        impl $crate::type_object::PySizedLayout<$name> for $layout {}
        impl<'a, $($generics,)*> $crate::derive_utils::PyBaseTypeUtils for $name {
            type Dict = $crate::pyclass_slots::PyClassDummySlot;
            type WeakRef = $crate::pyclass_slots::PyClassDummySlot;
            type LayoutAsBase = $crate::pycell::PyCellBase<$name>;
            type BaseNativeType = $name;
            type ThreadChecker = $crate::pyclass::ThreadCheckerStub<$crate::PyObject>;
        }
    }
}

#[macro_export]
macro_rules! pyobject_native_type {
    ($name: ty, $layout: path, $typeobject: expr, $module: expr, $checkfunction:path $(;$generics: ident)*) => {
        $crate::pyobject_native_type_core!($name, $layout, $typeobject, $module, $checkfunction $(;$generics)*);
        $crate::pyobject_native_type_sized!($name, $layout $(;$generics)*);
    };
    ($name: ty, $layout: path, $typeobject: expr, $checkfunction:path $(;$generics: ident)*) => {
        $crate::pyobject_native_type! {
            $name, $layout, $typeobject, Some("builtins"), $checkfunction $(;$generics)*
        }
    };
}

#[macro_export]
macro_rules! pyobject_native_var_type {
    ($name: ty, $typeobject: expr, $module: expr, $checkfunction:path $(;$generics: ident)*) => {
        $crate::pyobject_native_type_core!(
            $name, $crate::ffi::PyObject, $typeobject, Some("builtins"), $checkfunction $(;$generics)*);
    };
    ($name: ty, $typeobject: expr, $checkfunction: path $(;$generics: ident)*) => {
        $crate::pyobject_native_var_type! {
            $name, $typeobject, Some("builtins"), $checkfunction $(;$generics)*
        }
    };
}

// NOTE: This macro is not included in pyobject_native_type_base!
// because rust-numpy has a special implementation.
#[macro_export]
macro_rules! pyobject_native_type_extract {
    ($name: ty $(;$generics: ident)*) => {
        impl<'py, $($generics,)*> $crate::FromPyObject<'py> for &'py $name {
            fn extract(obj: &'py $crate::PyAny) -> $crate::PyResult<Self> {
                $crate::PyTryFrom::try_from(obj).map_err(Into::into)
            }
        }
    }
}

#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name: ty, $layout: path, $typeobject: expr,
     $module: expr $(, $checkfunction:path)? $(;$generics: ident)*) => {
        unsafe impl<$($generics,)*> $crate::type_object::PyTypeInfo for $name {
            type Type = ();
            type BaseType = $crate::PyAny;
            type Layout = $layout;
            type BaseLayout = $crate::ffi::PyObject;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<Self>;
            type AsRefTarget = Self;

            const NAME: &'static str = stringify!($name);
            const MODULE: Option<&'static str> = $module;

            #[inline]
            fn type_object_raw(_py: $crate::Python) -> *mut $crate::ffi::PyTypeObject {
                // Create a very short lived mutable reference and directly
                // cast it to a pointer: no mutable references can be aliasing
                // because we hold the GIL.
                unsafe { &mut $typeobject }
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

mod any;
mod boolobject;
mod bytearray;
mod bytes;
mod complex;
#[cfg(not(Py_LIMITED_API))]
mod datetime;
mod dict;
mod floatob;
mod function;
mod iterator;
mod list;
mod module;
mod num;
mod sequence;
mod set;
mod slice;
mod string;
mod tuple;
mod typeobject;
