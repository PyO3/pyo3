// TODO https://github.com/PyO3/pyo3/issues/5487
#![allow(clippy::undocumented_unsafe_blocks)]

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

#[doc(inline)]
pub use self::any::{PyAny, PyAnyMethods};
#[doc(inline)]
pub use self::boolobject::{PyBool, PyBoolMethods};
#[doc(inline)]
pub use self::bytearray::{PyByteArray, PyByteArrayMethods};
#[doc(inline)]
pub use self::bytes::{PyBytes, PyBytesMethods};
#[doc(inline)]
pub use self::capsule::{PyCapsule, PyCapsuleMethods};
#[doc(inline)]
pub use self::code::{PyCode, PyCodeMethods};
#[doc(inline)]
pub use self::complex::{PyComplex, PyComplexMethods};
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy, RustPython)))]
pub use self::context::PyContext;
#[doc(inline)]
pub use self::datetime::{PyDate, PyDateTime, PyDelta, PyTime, PyTzInfo, PyTzInfoAccess};
#[cfg(not(Py_LIMITED_API))]
#[doc(inline)]
pub use self::datetime::{PyDateAccess, PyDeltaAccess, PyTimeAccess};
#[cfg(not(any(PyPy, GraalPy, RustPython)))]
#[doc(inline)]
pub use self::dict::{items::PyDictItems, keys::PyDictKeys, values::PyDictValues};
#[doc(inline)]
pub use self::dict::{IntoPyDict, PyDict, PyDictMethods};
#[doc(inline)]
pub use self::ellipsis::PyEllipsis;
#[doc(inline)]
pub use self::float::{PyFloat, PyFloatMethods};
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
#[doc(inline)]
pub use self::frame::{PyFrame, PyFrameMethods};
#[cfg(Py_3_15)]
#[doc(inline)]
pub use self::frozendict::{PyFrozenDict, PyFrozenDictMethods};
#[doc(inline)]
pub use self::frozenset::{PyFrozenSet, PyFrozenSetMethods};
#[doc(inline)]
pub use self::function::{PyCFunction, PyFunction};
#[doc(inline)]
pub use self::genericalias::PyGenericAlias;
#[doc(inline)]
pub use self::int::PyInt;
#[doc(inline)]
pub use self::iterator::PyIterator;
#[doc(inline)]
pub use self::list::{PyList, PyListMethods};
#[doc(inline)]
pub use self::mapping::{PyMapping, PyMappingMethods};
#[doc(inline)]
pub use self::mappingproxy::{PyMappingProxy, PyMappingProxyMethods};
#[doc(inline)]
pub use self::memoryview::PyMemoryView;
#[doc(inline)]
pub use self::module::{PyModule, PyModuleMethods};
#[doc(inline)]
pub use self::none::PyNone;
#[doc(inline)]
pub use self::notimplemented::PyNotImplemented;
#[doc(inline)]
pub use self::pysuper::PySuper;
#[doc(inline)]
pub use self::range::{PyRange, PyRangeMethods};
#[doc(inline)]
pub use self::sequence::{PySequence, PySequenceMethods};
#[doc(inline)]
pub use self::set::{PySet, PySetMethods};
#[doc(inline)]
pub use self::slice::{PySlice, PySliceMethods};
#[doc(inline)]
pub use self::string::{PyString, PyStringMethods};
#[doc(inline)]
pub use self::traceback::{PyTraceback, PyTracebackMethods};
#[doc(inline)]
pub use self::tuple::{PyTuple, PyTupleMethods};
#[doc(inline)]
pub use self::typeobject::{PyType, PyTypeMethods};
#[doc(inline)]
pub use self::weakref::{PyWeakref, PyWeakrefMethods, PyWeakrefProxy, PyWeakrefReference};

/// Deprecated alias for [`crate::sync::PyMutex`].
#[cfg(all(not(Py_LIMITED_API), Py_3_13))]
#[deprecated(since = "0.30.0", note = "moved to `pyo3::sync::PyMutex`")]
pub type PyMutex<T> = crate::sync::PyMutex<T>;

/// Deprecated alias for [`crate::sync::PyMutexGuard`].
#[cfg(all(not(Py_LIMITED_API), Py_3_13))]
#[deprecated(since = "0.30.0", note = "moved to `pyo3::sync::PyMutexGuard`")]
pub type PyMutexGuard<'a, T> = crate::sync::PyMutexGuard<'a, T>;

/// Deprecated alias for [`capsule::CapsuleName`].
#[deprecated(
    since = "0.30.0",
    note = "moved to `pyo3::types::capsule::CapsuleName`"
)]
pub type CapsuleName = capsule::CapsuleName;

/// Deprecated alias for [`code::PyCodeInput`].
#[deprecated(since = "0.30.0", note = "moved to `pyo3::types::code::PyCodeInput`")]
pub type PyCodeInput = code::PyCodeInput;

/// Deprecated alias for [`frozenset::PyFrozenSetBuilder`].
#[deprecated(
    since = "0.30.0",
    note = "moved to `pyo3::types::frozenset::PyFrozenSetBuilder`"
)]
pub type PyFrozenSetBuilder<'py> = frozenset::PyFrozenSetBuilder<'py>;

/// Deprecated alias for [`iterator::PySendResult`].
#[cfg(all(not(PyPy), Py_3_10))]
#[deprecated(
    since = "0.30.0",
    note = "moved to `pyo3::types::iterator::PySendResult`"
)]
pub type PySendResult<'py> = iterator::PySendResult<'py>;

/// Deprecated alias for [`slice::PySliceIndices`].
#[deprecated(
    since = "0.30.0",
    note = "moved to `pyo3::types::slice::PySliceIndices`"
)]
pub type PySliceIndices = slice::PySliceIndices;

/// Deprecated alias for [`string::PyStringData`].
#[cfg(not(Py_LIMITED_API))]
#[deprecated(
    since = "0.30.0",
    note = "moved to `pyo3::types::string::PyStringData`"
)]
pub type PyStringData<'a> = string::PyStringData<'a>;

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
///
/// # pub fn main() -> PyResult<()> {
/// Python::attach(|py| {
///     let dict = py.eval(c"{'a':'b', 'c':'d'}", None, None)?.cast_into::<PyDict>()?;
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
    // TODO: we might want to remove this `iter` module and instead just let the
    // iterators be imported directly from their respective modules.

    pub use super::dict::BoundDictIterator;
    #[cfg(Py_3_15)]
    pub use super::frozendict::BoundFrozenDictIterator;
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
/// This is used to provide the [`Deref<Target = Bound<'_, PyAny>>`](core::ops::Deref)
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

/// Helper for defining the `$typeobject` argument for other macros in this module.
///
/// # Safety
///
/// - `$typeobject` must be a known `static mut PyTypeObject`
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_static_type_object(
    ($typeobject:expr) => {
        |_py| &raw mut $typeobject
    };
);

/// Adds a TYPE_HINT constant if the `experimental-inspect`  feature is enabled.
#[cfg(not(feature = "experimental-inspect"))]
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_type_info_type_hint(
    ($module:expr, $name:expr) => {};
);

#[cfg(feature = "experimental-inspect")]
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_type_info_type_hint(
    ($module:expr, $name:expr) => {
        const TYPE_HINT: $crate::inspect::PyStaticExpr = $crate::type_hint_identifier!($module, $name);
    };
);

/// Implements the `PyTypeInfo` trait for a native Python type.
///
/// # Safety
///
/// - `$typeobject` must be a function that produces a valid `*mut PyTypeObject`
/// - `$checkfunction` must be a function that accepts arbitrary `*mut PyObject` and returns true /
///   false according to whether the object is an instance of the type from `$typeobject`
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name:ty, $typeobject:expr, $type_hint_module:expr, $type_hint_name:expr, $module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        // SAFETY: macro caller has upheld the safety contracts
        unsafe impl<$($generics,)*> $crate::type_object::PyTypeInfo for $name {
            const NAME: &'static str = stringify!($name);
            const MODULE: ::core::option::Option<&'static str> = $module;
            $crate::pyobject_type_info_type_hint!($type_hint_module, $type_hint_name);

            #[inline]
            #[allow(clippy::redundant_closure_call)]
            fn type_object_raw(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                $typeobject(py)
            }

            $(
                #[inline]
                fn is_type_of(obj: &$crate::Bound<'_, $crate::PyAny>) -> bool {
                    #[allow(unused_unsafe, reason = "not all `$checkfunction` are unsafe fn")]
                    // SAFETY: `$checkfunction` is being called with a valid `PyObject` pointer
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

/// Declares all of the boilerplate for Python types.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_core {
    ($name:ty, $typeobject:expr, $type_hint_module:expr, $type_hint_name:expr, #module=$module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_named!($name $(;$generics)*);
        $crate::pyobject_native_type_info!($name, $typeobject, $type_hint_module, $type_hint_name, $module $(, #checkfunction=$checkfunction)? $(;$generics)*);
    };
    ($name:ty, $typeobject:expr, $type_hint_module:expr, $type_hint_name:expr, #module=$module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name, $typeobject, $type_hint_module, $type_hint_name, #module=$module $(, #checkfunction=$checkfunction)? $(;$generics)*);
    };
    ($name:ty, $typeobject:expr, $type_hint_module:expr, $type_hint_name:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name, $typeobject, $type_hint_module, $type_hint_name, #module=::core::option::Option::Some("builtins") $(, #checkfunction=$checkfunction)? $(;$generics)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_subclassable_native_type {
    ($name:ty, $layout:path $(;$generics:ident)*) => {
        #[cfg(not(Py_LIMITED_API))]
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::impl_::pycell::PyClassObjectBase<$layout>;
            type BaseNativeType = $name;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> = $crate::impl_::pycell::PyStaticClassObject<T>;
        }

        #[cfg(all(Py_3_12, Py_LIMITED_API))]
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::impl_::pycell::PyVariableClassObjectBase;
            type BaseNativeType = Self;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> = $crate::impl_::pycell::PyVariableClassObject<T>;
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_sized {
    ($name:ty, $layout:path $(;$generics:ident)*) => {
        // SAFETY: native objects are valid
        unsafe impl $crate::type_object::PyLayout<$name> for $layout {}
        impl $crate::type_object::PySizedLayout<$name> for $layout {}
    };
}

/// Declares all of the boilerplate for Python types which can be inherited from (because the exact
/// Python layout is known).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type {
    ($name:ty, $layout:path, $typeobject:expr, $type_hint_module:expr, $type_hint_name:expr $(, #module=$module:expr)? $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        $crate::pyobject_native_type_core!($name, $typeobject, $type_hint_module, $type_hint_name $(, #module=$module)? $(, #checkfunction=$checkfunction)? $(;$generics)*);
        // To prevent inheriting native types with ABI3
        #[cfg(not(Py_LIMITED_API))]
        $crate::pyobject_native_type_sized!($name, $layout $(;$generics)*);
    };
}

pub(crate) mod any;
pub(crate) mod boolobject;
pub(crate) mod bytearray;
pub(crate) mod bytes;
pub mod capsule;
pub mod code;
pub(crate) mod complex;
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy, RustPython)))]
mod context;
pub mod datetime;
pub mod dict;
mod ellipsis;
pub(crate) mod float;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
mod frame;
#[cfg(Py_3_15)]
pub mod frozendict;
pub mod frozenset;
mod function;
pub(crate) mod genericalias;
mod int;
pub mod iterator;
pub mod list;
pub(crate) mod mapping;
pub mod mappingproxy;
mod memoryview;
pub(crate) mod module;
mod none;
mod notimplemented;
mod pysuper;
pub(crate) mod range;
pub(crate) mod sequence;
pub mod set;
pub mod slice;
pub mod string;
pub(crate) mod traceback;
pub mod tuple;
pub(crate) mod typeobject;
pub(crate) mod weakref;
