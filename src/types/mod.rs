//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

pub use self::any::{PyAny, PyAnyMethods};
pub use self::boolobject::{PyBool, PyBoolMethods};
pub use self::bytearray::{PyByteArray, PyByteArrayMethods};
pub use self::bytes::{PyBytes, PyBytesMethods};
pub use self::capsule::{PyCapsule, PyCapsuleMethods};
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
pub use self::code::PyCode;
pub use self::complex::{PyComplex, PyComplexMethods};
#[allow(deprecated)]
pub use self::datetime::{
    timezone_utc, PyDate, PyDateTime, PyDelta, PyTime, PyTzInfo, PyTzInfoAccess,
};
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::{PyDateAccess, PyDeltaAccess, PyTimeAccess};
pub use self::dict::{IntoPyDict, PyDict, PyDictMethods};
#[cfg(not(any(PyPy, GraalPy)))]
pub use self::dict::{PyDictItems, PyDictKeys, PyDictValues};
pub use self::ellipsis::PyEllipsis;
pub use self::float::{PyFloat, PyFloatMethods};
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
pub use self::frame::PyFrame;
pub use self::frozenset::{PyFrozenSet, PyFrozenSetBuilder, PyFrozenSetMethods};
pub use self::function::PyCFunction;
#[cfg(all(not(Py_LIMITED_API), not(all(PyPy, not(Py_3_8)))))]
pub use self::function::PyFunction;
#[cfg(Py_3_9)]
pub use self::genericalias::PyGenericAlias;
pub use self::iterator::PyIterator;
pub use self::list::{PyList, PyListMethods};
pub use self::mapping::{PyMapping, PyMappingMethods};
pub use self::mappingproxy::PyMappingProxy;
pub use self::memoryview::PyMemoryView;
pub use self::module::{PyModule, PyModuleMethods};
pub use self::none::PyNone;
pub use self::notimplemented::PyNotImplemented;
pub use self::num::PyInt;
#[cfg(not(any(PyPy, GraalPy)))]
pub use self::pysuper::PySuper;
pub use self::sequence::{PySequence, PySequenceMethods};
pub use self::set::{PySet, PySetMethods};
pub use self::slice::{PySlice, PySliceIndices, PySliceMethods};
#[cfg(not(Py_LIMITED_API))]
pub use self::string::PyStringData;
pub use self::string::{PyString, PyStringMethods};
pub use self::traceback::{PyTraceback, PyTracebackMethods};
pub use self::tuple::{PyTuple, PyTupleMethods};
pub use self::typeobject::{PyType, PyTypeMethods};
pub use self::weakref::{PyWeakref, PyWeakrefMethods, PyWeakrefProxy, PyWeakrefReference};

/// Iteration over Python collections.
///
/// When working with a Python collection, one approach is to convert it to a Rust collection such
/// as `Vec` or `HashMap`. However this is a relatively expensive operation. If you just want to
/// visit all their items, consider iterating over the collections directly:
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyDict;
/// use pyo3::ffi::c_str;
///
/// # pub fn main() -> PyResult<()> {
/// Python::with_gil(|py| {
///     let dict = py.eval(c_str!("{'a':'b', 'c':'d'}"), None, None)?.downcast_into::<PyDict>()?;
///
///     for (key, value) in &dict {
///         println!("key: {}, value: {}", key, value);
///     }
///
///     Ok(())
/// })
/// # }
///  ```
///
/// If PyO3 detects that the collection is mutated during iteration, it will panic.
///
/// These iterators use Python's C-API directly. However in certain cases, like when compiling for
/// the Limited API and PyPy, the underlying structures are opaque and that may not be possible.
/// In these cases the iterators are implemented by forwarding to [`PyIterator`].
pub mod iter {
    pub use super::dict::BoundDictIterator;
    pub use super::frozenset::BoundFrozenSetIterator;
    pub use super::list::BoundListIterator;
    pub use super::set::BoundSetIterator;
    pub use super::tuple::{BorrowedTupleIterator, BoundTupleIterator};
}

/// Python objects that have a base type.
///
/// This marks types that can be upcast into a [`PyAny`] and used in its place.
/// This essentially includes every Python object except [`PyAny`] itself.
///
/// This is used to provide the [`Deref<Target = Bound<'_, PyAny>>`](std::ops::Deref)
/// implementations for [`Bound<'_, T>`](crate::Bound).
///
/// Users should not need to implement this trait directly. It's implementation
/// is provided by the [`#[pyclass]`](macro@crate::pyclass) attribute.
///
/// ## Note
/// This is needed because the compiler currently tries to figure out all the
/// types in a deref-chain before starting to look for applicable method calls.
/// So we need to prevent [`Bound<'_, PyAny`](crate::Bound) dereferencing to
/// itself in order to avoid running into the recursion limit. This trait is
/// used to exclude this from our blanket implementation. See [this Rust
/// issue][1] for more details. If the compiler limitation gets resolved, this
/// trait will be removed.
///
/// [1]: https://github.com/rust-lang/rust/issues/19509
pub trait DerefToPyAny {
    // Empty.
}

// Implementations core to all native types except for PyAny (because they don't
// make sense on PyAny / have different implementations).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_named (
    ($name:ty $(;$generics:ident)*) => {
        impl $crate::types::DerefToPyAny for $name {}
    };
);

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name:ty, $module:expr, $opaque:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        unsafe impl<$($generics,)*> $crate::type_object::PyTypeInfo for $name {
            const NAME: &'static str = stringify!($name);
            const MODULE: ::std::option::Option<&'static str> = $module;
            const OPAQUE: bool = $opaque;

            #[inline]
            fn type_object_raw(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                // provided by pyobject_native_type_object_methods!()
                Self::type_object_raw_impl(py)
            }

            #[inline]
            fn try_get_type_object_raw() -> ::std::option::Option<*mut $crate::ffi::PyTypeObject> {
                // provided by pyobject_native_type_object_methods!()
                Self::try_get_type_object_raw_impl()
            }

            $(
                #[inline]
                fn is_type_of(obj: &$crate::Bound<'_, $crate::PyAny>) -> bool {
                    #[allow(unused_unsafe)]
                    unsafe { $checkfunction(obj.as_ptr()) > 0 }
                }
            )?
        }

        impl $name {
            #[doc(hidden)]
            pub const _PYO3_DEF: $crate::impl_::pymodule::AddTypeToModule<Self> = $crate::impl_::pymodule::AddTypeToModule::new();

            #[allow(dead_code)]
            #[doc(hidden)]
            pub const _PYO3_INTROSPECTION_ID: &'static str = concat!(stringify!($module), stringify!($name));
        }
    };
);

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_marker(
    ($name:ty) => {
        unsafe impl $crate::type_object::PyNativeType for $name {}
    }
);

/// Declares all of the boilerplate for Python types.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_core {
    ($name:ty, #module=$module:expr, #opaque=$opaque:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_named!($name $(;$generics)*);
        $crate::pyobject_native_type_marker!($name);
        $crate::pyobject_native_type_info!(
            $name,
            $module,
            $opaque
            $(, #checkfunction=$checkfunction)?
            $(;$generics)*
        );
    };
    ($name:ty, #module=$module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!(
            $name,
            #module=$module,
            #opaque=false
            $(, #checkfunction=$checkfunction)?
            $(;$generics)*
        );
    };
    ($name:ty $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!(
            $name,
            #module=::std::option::Option::Some("builtins"),
            #opaque=false
            $(, #checkfunction=$checkfunction)?
            $(;$generics)*
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_subclassable_native_type {
    ($name:ty, $layout:path $(;$generics:ident)*) => {
        #[cfg(not(Py_LIMITED_API))]
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type StaticLayout = $crate::impl_::pycell::PyStaticNativeLayout<$layout>;
            type BaseNativeType = $name;
            type RecursiveOperations = $crate::impl_::pycell::PyNativeTypeRecursiveOperations<Self>;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::borrow_checker::ImmutableClass;
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_sized {
    ($name:ty, $layout:path $(;$generics:ident)*) => {
        unsafe impl $crate::type_object::PyLayout<$name> for $layout {}
        impl $crate::type_object::PySizedLayout<$name> for $layout {}
    };
}

/// Declares all of the boilerplate for Python types which can be inherited from (because the exact
/// Python layout is known).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type {
    ($name:ty, $layout:path $(, #module=$module:expr)? $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name $(, #module=$module)? $(, #checkfunction=$checkfunction)? $(;$generics)*);
        // To prevent inheriting native types with ABI3
        #[cfg(not(Py_LIMITED_API))]
        $crate::pyobject_native_type_sized!($name, $layout $(;$generics)*);
    };
}

/// Implement methods for obtaining the type object associated with a native type.
/// These methods are referred to in `pyobject_native_type_info` for implementing `PyTypeInfo`.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_object_methods {
    // the type object is not known statically and so must be created (once) with the GIL held
    ($name:ty, #create=$create_type_object:expr) => {
        impl $name {
            fn type_object_cell() -> &'static $crate::sync::GILOnceCell<$crate::Py<$crate::types::PyType>> {
                static TYPE_OBJECT: $crate::sync::GILOnceCell<$crate::Py<$crate::types::PyType>> =
                    $crate::sync::GILOnceCell::new();
                &TYPE_OBJECT
            }

            #[allow(clippy::redundant_closure_call)]
            fn type_object_raw_impl(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                Self::type_object_cell()
                    .get_or_init(py, || $create_type_object(py))
                    .as_ptr()
                    .cast::<$crate::ffi::PyTypeObject>()
            }

            fn try_get_type_object_raw_impl() -> ::std::option::Option<*mut $crate::ffi::PyTypeObject> {
                unsafe {
                    Self::type_object_cell().get_raw().map(|obj| { (*obj).as_ptr().cast() })
                }
            }
        }
    };
    // the type object can be created without holding the GIL
    ($name:ty, #get=$get_type_object:expr) => {
        impl $name {
            fn type_object_raw_impl(_py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                Self::try_get_type_object_raw_impl().expect("type object is None when it should be Some")
            }

            #[allow(clippy::redundant_closure_call)]
            fn try_get_type_object_raw_impl() -> ::std::option::Option<*mut $crate::ffi::PyTypeObject> {
                Some($get_type_object())
            }
        }
    };
    // the type object is imported from a module
    ($name:ty, #import_module=$import_module:expr, #import_name=$import_name:expr) => {
        $crate::pyobject_native_type_object_methods!($name, #create=|py: $crate::Python<'_>| {
            let module = stringify!($import_module);
            let name = stringify!($import_name);
            || -> $crate::PyResult<$crate::Py<$crate::types::PyType>> {
                use $crate::types::PyAnyMethods;
                $crate::PyResult::Ok(py.import(module)?.getattr(name)?.downcast_into()?.unbind())
            }()
            .unwrap_or_else(|e| ::std::panic!("failed to import {}.{}: {}", module, name, e))
        });
    };
    // the type object is known statically
    ($name:ty, #global=$ffi_type_object:path) => {
        $crate::pyobject_native_type_object_methods!($name, #get=|| {
            #[allow(unused_unsafe)] // https://github.com/rust-lang/rust/pull/125834
            unsafe { ::std::ptr::addr_of_mut!($ffi_type_object) }
        });
    };
    // the type object is known statically
    ($name:ty, #global_ptr=$ffi_type_object:path) => {
        $crate::pyobject_native_type_object_methods!($name, #get=|| {
            unsafe { $ffi_type_object.cast::<$crate::ffi::PyTypeObject>() }
        });
    };
}

pub(crate) mod any;
pub(crate) mod boolobject;
pub(crate) mod bytearray;
pub(crate) mod bytes;
pub(crate) mod capsule;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
mod code;
pub(crate) mod complex;
pub(crate) mod datetime;
pub(crate) mod dict;
mod ellipsis;
pub(crate) mod float;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
mod frame;
pub(crate) mod frozenset;
mod function;
#[cfg(Py_3_9)]
pub(crate) mod genericalias;
pub(crate) mod iterator;
pub(crate) mod list;
pub(crate) mod mapping;
pub(crate) mod mappingproxy;
mod memoryview;
pub(crate) mod module;
mod none;
mod notimplemented;
mod num;
#[cfg(not(any(PyPy, GraalPy)))]
mod pysuper;
pub(crate) mod sequence;
pub(crate) mod set;
pub(crate) mod slice;
pub(crate) mod string;
pub(crate) mod traceback;
pub(crate) mod tuple;
pub(crate) mod typeobject;
pub(crate) mod weakref;
