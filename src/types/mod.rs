// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the python interpreter such as `int`, `str` and `tuple`

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
    ($name: ty $(,$type_param: ident)*) => {
        impl<$($type_param,)*> ::std::convert::AsRef<$crate::types::PyAny> for $name {
            #[inline]
            fn as_ref(&self) -> &$crate::types::PyAny {
                unsafe{&*(self as *const $name as *const $crate::types::PyAny)}
            }
        }

        unsafe impl<$($type_param,)*> $crate::PyNativeType for $name {}

        impl<$($type_param,)*> $crate::AsPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$($type_param,)*> PartialEq for $name {
            #[inline]
            fn eq(&self, o: &$name) -> bool {
                use $crate::AsPyPointer;

                self.as_ptr() == o.as_ptr()
            }
        }
    };
);

#[macro_export]
macro_rules! pyobject_native_type {
    ($name: ty, $layout: path, $typeobject: expr, $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl $crate::type_object::PyObjectLayout<$name> for $layout {}
        impl $crate::type_object::PyObjectSizedLayout<$name> for $layout {}
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_type_convert!($name, $layout, $typeobject, $module, $checkfunction $(,$type_param)*);

        impl<'a, $($type_param,)*> ::std::convert::From<&'a $name> for &'a $crate::types::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::types::PyAny)}
            }
        }
    };
    ($name: ty, $layout: path, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_type! {
            $name, $layout, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
}

#[macro_export]
macro_rules! pyobject_native_var_type {
    ($name: ty, $typeobject: expr, $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl $crate::type_object::PyObjectLayout<$name> for $crate::ffi::PyObject {}
        pyobject_native_type_named!($name $(,$type_param)*);
        pyobject_native_type_convert!($name, $crate::ffi::PyObject,
                                      $typeobject, $module, $checkfunction $(,$type_param)*);
        impl<'a, $($type_param,)*> ::std::convert::From<&'a $name> for &'a $crate::types::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::types::PyAny)}
            }
        }
    };
    ($name: ty, $typeobject: expr, $checkfunction: path $(,$type_param: ident)*) => {
        pyobject_native_var_type! {
            $name, $typeobject, Some("builtins"), $checkfunction $(,$type_param)*
        }
    };
}

#[macro_export]
macro_rules! pyobject_native_type_convert(
    ($name: ty, $layout: path, $typeobject: expr,
     $module: expr, $checkfunction: path $(,$type_param: ident)*) => {
        impl<$($type_param,)*> $crate::type_object::PyTypeInfo for $name {
            type Type = ();
            type BaseType = $crate::types::PyAny;
            type ConcreteLayout = $layout;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<Self>;

            const NAME: &'static str = stringify!($name);
            const MODULE: Option<&'static str> = $module;

            #[inline]
            unsafe fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                &mut $typeobject
            }

            #[allow(unused_unsafe)]
            fn is_instance(ptr: &$crate::types::PyAny) -> bool {
                use $crate::AsPyPointer;

                unsafe { $checkfunction(ptr.as_ptr()) > 0 }
            }
        }

        unsafe impl<$($type_param,)*> $crate::type_object::PyTypeObject for $name {
            fn init_type() -> std::ptr::NonNull<$crate::ffi::PyTypeObject> {
                unsafe {
                    std::ptr::NonNull::new_unchecked(
                        <Self as $crate::type_object::PyTypeInfo>::type_object() as *mut _
                    )
                }
            }
        }

        impl<$($type_param,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe {$crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())}
            }
        }

        impl<$($type_param,)*> ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                use $crate::ObjectProtocol;
                let s = self.repr().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$($type_param,)*> ::std::fmt::Display for $name {
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
