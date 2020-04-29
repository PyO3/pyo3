// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

pub use self::any::PyAny;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
pub use self::complex::PyComplex;
pub use self::datetime::PyDeltaAccess;
pub use self::datetime::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyTime, PyTimeAccess, PyTzInfo,
};
pub use self::dict::{IntoPyDict, PyDict};
pub use self::floatob::PyFloat;
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::module::PyModule;
pub use self::num::PyLong;
pub use self::num::PyLong as PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
pub use self::string::{PyString, PyString as PyUnicode};
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;

#[macro_export]
macro_rules! pyobject_native_type_common (
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >) => {
        unsafe impl<'py $(, $type_param $(: $bound)?)*> $crate::PyNativeType<'py> for $name<'py $(,$type_param)*> {}

        impl<'py $(, $type_param $(: $bound)?)*> $crate::AsPyPointer for $name<'py $(,$type_param)*> {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<'py $(, $type_param $(: $bound)?)*> $crate::IntoPyPointer for $name<'py $(,$type_param)*> {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                use $crate::AsPyPointer;
                let ptr = self.as_ptr();
                std::mem::forget(self);
                ptr
            }
        }

        impl<'py $(, $type_param $(: $bound)?)*> PartialEq for $name<'py $(,$type_param)*> {
            #[inline]
            fn eq(&self, o: &$name<'py $(,$type_param)*>) -> bool {
                use $crate::AsPyPointer;

                self.as_ptr() == o.as_ptr()
            }
        }

        impl<'py $(, $type_param $(: $bound)?)*> $crate::ToPyObject for $name<'py $(,$type_param)*>
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.as_ptr())}
            }
        }

        impl<'py $(, $type_param $(: $bound)?)*> std::convert::From<$name<'py $(,$type_param)*>> for $crate::PyObject
        {
            fn from(ob: $name<'py $(,$type_param)*>) -> Self {
                use $crate::{IntoPyPointer, PyNativeType};
                unsafe { Self::from_owned_ptr(ob.py(), ob.into_ptr()) }
            }
        }
    }
);

#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >) => {
        impl<'py $(, $type_param $(: $bound)?)*> ::std::fmt::Debug for $name<'py $(,$type_param)*> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.repr().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<'py $(, $type_param $(: $bound)?)*> ::std::fmt::Display for $name<'py $(,$type_param)*> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.str().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type {
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $layout: path, $typeobject: expr, $module: expr, $checkfunction: path) => {
        impl<'py $(, $type_param $(: $bound)?)*> $crate::type_object::PySizedLayout<'py, $name<'py $(,$type_param)*>> for $layout {}
        impl<'py $(, $type_param $(: $bound)?)*> $crate::derive_utils::PyBaseTypeUtils<'py> for $name<'py $(,$type_param)*> {
            type Dict = $crate::pyclass_slots::PyClassDummySlot;
            type WeakRef = $crate::pyclass_slots::PyClassDummySlot;
            type LayoutAsBase = $crate::pycell::PyCellBase<'py, Self>;
            type BaseNativeType = Self;
        }
        pyobject_native_type_named!($name<'py $(, $type_param $(: $bound)?)*>);
        pyobject_native_newtype!($name<'py $(, $type_param $(: $bound)?)*>);
        pyobject_native_type_info!($name<'py $(, $type_param $(: $bound)?)*>, $layout, $typeobject, $module, $checkfunction);
        pyobject_native_type_extract!($name<'py $(, $type_param $(: $bound)?)*>);
    };
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $layout: path, $typeobject: expr, $checkfunction: path) => {
        pyobject_native_type! {
            $name<'py $(, $type_param $(: $bound)?)*>, $layout, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
}

#[macro_export]
macro_rules! pyobject_native_var_type {
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $typeobject: expr, $module: expr, $checkfunction: path) => {
        pyobject_native_newtype!($name<'py $(, $type_param $(: $bound)?)*>);
        pyobject_native_type_named!($name<'py $(, $type_param $(: $bound)?)*>);
        pyobject_native_type_info!($name<'py $(, $type_param $(: $bound)?)*>, $crate::ffi::PyObject,
                                   $typeobject, $module, $checkfunction);
        pyobject_native_type_extract!($name<'py $(, $type_param $(: $bound)?)*>);
    };
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $typeobject: expr, $checkfunction: path) => {
        pyobject_native_var_type! {
            $name<'py $(, $type_param $(: $bound)?)*>, $typeobject, Some("builtins"), $checkfunction
        }
    };
}

// NOTE: This macro is not included in pyobject_native_newtype!
// because rust-numpy has a special implementation.
#[macro_export]
macro_rules! pyobject_native_type_extract {
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >) => {
        impl<'a, 'py $(, $type_param $(: $bound)?)*> $crate::FromPyObject<'a, 'py> for &'a $name<'py $(,$type_param)*> {
            fn extract(obj: &'a $crate::PyAny<'py>) -> $crate::PyResult<Self> {
                $crate::PyTryFrom::try_from(obj).map_err(Into::into)
            }
        }
    }
}

#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $layout: path, $typeobject: expr,
     $module: expr, $checkfunction: path) => {
        unsafe impl<'py $(, $type_param $(: $bound)?)*> $crate::type_object::PyLayout<'py, $name<'py $(,$type_param)*>> for $layout {}

        unsafe impl<'py $(, $type_param $(: $bound)?)*> $crate::type_object::PyTypeInfo<'py> for $name<'py $(,$type_param)*> {
            type Type = ();
            type BaseType = $crate::PyAny<'py>;
            type Layout = $layout;
            type BaseLayout = ffi::PyObject;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<'py, Self>;
            type AsRefTarget = Self;

            const NAME: &'static str = stringify!($name);
            const MODULE: Option<&'static str> = $module;

            #[inline]
            fn type_object() -> &'static $crate::ffi::PyTypeObject {
                unsafe{ &$typeobject }
            }

            #[allow(unused_unsafe)]
            fn is_instance(ptr: &$crate::PyAny) -> bool {
                use $crate::AsPyPointer;
                unsafe { $checkfunction(ptr.as_ptr()) > 0 }
            }
        }
    };

    ($name: ident < 'py $( ,$type_param: ident $(: $bound: path)? )* >, $layout: path, $typeobject: expr, $checkfunction: path) => {
        pyobject_native_type_info! {
            $name<'py $(, $type_param $(: $bound)?)*>, $layout, $typeobject, Some("builtins"), $checkfunction
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_newtype(
    ($name: ident < 'py $( ,$type_param: ident)* > ) => {
        pyobject_native_type_common!($name<'py $(,$type_param)*>);

        impl<'a, 'py, $($type_param,)*> ::std::convert::From<&'a $name<'py $(,$type_param)*>> for &'a $crate::PyAny<'a> {
            fn from(ob: &'a $name<'py $(,$type_param)*>) -> Self {
                unsafe{&*(ob as *const _ as *const $crate::PyAny)}
            }
        }

        impl<'py, $($type_param,)*> ::std::convert::AsRef<$crate::PyAny<'py>> for $name<'py $(,$type_param)*> {
            #[inline]
            fn as_ref(&self) -> &$crate::PyAny<'py> {
                &self.0
            }
        }

        impl<'py, $($type_param,)*> ::std::ops::Deref for $name<'py $(,$type_param)*> {
            type Target = $crate::PyAny<'py>;

            #[inline]
            fn deref(&self) -> &$crate::PyAny<'py> {
                &self.0
            }
        }

        unsafe impl<'py $(,$type_param)*> $crate::FromPyPointer<'py> for $name<'py $(,$type_param)*>
        {
            unsafe fn from_owned_ptr_or_opt(py: $crate::Python<'py>, ptr: *mut $crate::ffi::PyObject) -> Option<Self> {
                ::std::ptr::NonNull::new(ptr).map(|p| Self(PyAny::from_non_null(py, p)))
            }
            unsafe fn from_borrowed_ptr_or_opt(
                py: $crate::Python<'py>,
                ptr: *mut $crate::ffi::PyObject,
            ) -> Option<&'py Self> {
                use $crate::type_object::PyDowncastImpl;
                ::std::ptr::NonNull::new(ptr).map(|p| Self::unchecked_downcast($crate::gil::register_borrowed(py, p)))
            }
        }

        impl<'py $(,$type_param)*> Clone for $name<'py $(,$type_param)*> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
    };
);

mod any;
mod boolobject;
mod bytearray;
mod bytes;
mod complex;
mod datetime;
mod dict;
mod floatob;
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
