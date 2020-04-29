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
macro_rules! pyobject_native_type_named (
    ($name: ident $(,$type_param: ident)*) => {
        unsafe impl<'py, $($type_param,)*> $crate::PyNativeType<'py> for $name<'py> {}

        impl<$($type_param,)*> $crate::AsPyPointer for $name<'_> {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$($type_param,)*> $crate::IntoPyPointer for $name<'_> {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                self.into_non_null().as_ptr()
            }
        }

        impl<$($type_param,)*> PartialEq for $name<'_> {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                use $crate::AsPyPointer;

                self.as_ptr() == o.as_ptr()
            }
        }

        impl<$($type_param,)*> $crate::ToPyObject for $name<'_>
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.as_ptr())}
            }
        }

        impl std::convert::From<$name<'_>> for $crate::PyObject
        {
            fn from(ob: $name<'_>) -> Self {
                Self::from_non_null(ob.into_non_null())
            }
        }

        impl<$($type_param,)*> ::std::fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.repr().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($type_param,)*> ::std::fmt::Display for $name<'_> {
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
    ($name: ident, $layout: path, $typeobject: expr, $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl<'py> $crate::type_object::PySizedLayout<'py, $name<'py>> for $layout {}
        impl<'py> $crate::derive_utils::PyBaseTypeUtils<'py> for $name<'py> {
            type Dict = $crate::pyclass_slots::PyClassDummySlot;
            type WeakRef = $crate::pyclass_slots::PyClassDummySlot;
            type LayoutAsBase = $crate::pycell::PyCellBase<'py, Self>;
            type BaseNativeType = Self;
        }
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_newtype!($name $(,$type_param)*);
        pyobject_native_type_info!($name, $layout, $typeobject, $module, $checkfunction $(,$type_param)*);
        pyobject_native_type_extract!($name $(,$type_param)*);


    };
    ($name: ident, $layout: path, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type! {
            $name, $layout, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
}

#[macro_export]
macro_rules! pyobject_native_var_type {
    ($name: ident, $typeobject: expr, $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_newtype!($name $(,$type_param)*);
        pyobject_native_type_info!($name, $crate::ffi::PyObject,
                                   $typeobject, $module, $checkfunction $(,$type_param)*);
        pyobject_native_type_extract!($name $(,$type_param)*);
    };
    ($name: ident, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_var_type! {
            $name, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
}

// NOTE: This macro is not included in pyobject_native_newtype!
// because rust-numpy has a special implementation.
macro_rules! pyobject_native_type_extract {
    ($name: ident $(,$type_param: ident)*) => {
        impl<'a, 'py, $($type_param,)*> $crate::FromPyObject<'a, 'py> for &'a $name<'py> {
            fn extract(obj: &'a $crate::PyAny<'py>) -> $crate::PyResult<Self> {
                $crate::PyTryFrom::try_from(obj).map_err(Into::into)
            }
        }
    }
}

#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name: ident, $layout: path, $typeobject: expr,
     $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        unsafe impl<'py> $crate::type_object::PyLayout<'py, $name<'py>> for $layout {}

        unsafe impl<'py, $($type_param,)*> $crate::type_object::PyTypeInfo<'py> for $name<'py> {
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

    ($name: ident, $layout: path, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type_info! {
            $name, $layout, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_newtype(
    ($name: ident $(,$type_param: ident)*) => {
        impl<'a, $($type_param,)*> ::std::convert::From<&'a $name<'a>> for &'a $crate::PyAny<'a> {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::PyAny)}
            }
        }

        impl Clone for $name<'_> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl<'a, $($type_param,)*> ::std::convert::AsRef<$crate::PyAny<'a>> for $name<'a> {
            #[inline]
            fn as_ref(&self) -> &$crate::PyAny<'a> {
                &self.0
            }
        }

        impl<'a, $($type_param,)*> ::std::ops::Deref for $name<'a> {
            type Target = $crate::PyAny<'a>;

            #[inline]
            fn deref(&self) -> &$crate::PyAny<'a> {
                &self.0
            }
        }

        unsafe impl<'p> $crate::FromPyPointer<'p> for $name<'p>
        {
            unsafe fn from_owned_ptr_or_opt(py: $crate::Python<'p>, ptr: *mut $crate::ffi::PyObject) -> Option<Self> {
                ::std::ptr::NonNull::new(ptr).map(|p| Self(PyAny::from_non_null(py, p)))
            }
            unsafe fn from_borrowed_ptr_or_opt(
                py: $crate::Python<'p>,
                ptr: *mut $crate::ffi::PyObject,
            ) -> Option<&'p Self> {
                use $crate::type_object::PyDowncastImpl;
                ::std::ptr::NonNull::new(ptr).map(|p| Self::unchecked_downcast($crate::gil::register_borrowed(py, p)))
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
