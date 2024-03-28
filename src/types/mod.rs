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
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::timezone_utc;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::{
    timezone_utc_bound, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime,
    PyTimeAccess, PyTzInfo, PyTzInfoAccess,
};
pub use self::dict::{IntoPyDict, PyDict, PyDictMethods};
#[cfg(not(any(PyPy, GraalPy)))]
pub use self::dict::{PyDictItems, PyDictKeys, PyDictValues};
pub use self::ellipsis::PyEllipsis;
pub use self::float::{PyFloat, PyFloatMethods};
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
pub use self::frame::PyFrame;
pub use self::frozenset::{PyFrozenSet, PyFrozenSetBuilder, PyFrozenSetMethods};
pub use self::function::PyCFunction;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
pub use self::function::PyFunction;
pub use self::iterator::PyIterator;
pub use self::list::{PyList, PyListMethods};
pub use self::mapping::{PyMapping, PyMappingMethods};
pub use self::memoryview::PyMemoryView;
pub use self::module::{PyModule, PyModuleMethods};
pub use self::none::PyNone;
pub use self::notimplemented::PyNotImplemented;
pub use self::num::PyLong;
pub use self::num::PyLong as PyInt;
#[cfg(not(any(PyPy, GraalPy)))]
pub use self::pysuper::PySuper;
pub use self::sequence::{PySequence, PySequenceMethods};
pub use self::set::{PySet, PySetMethods};
pub use self::slice::{PySlice, PySliceIndices, PySliceMethods};
#[cfg(not(Py_LIMITED_API))]
pub use self::string::PyStringData;
pub use self::string::{PyString, PyString as PyUnicode, PyStringMethods};
pub use self::traceback::{PyTraceback, PyTracebackMethods};
pub use self::tuple::{PyTuple, PyTupleMethods};
pub use self::typeobject::{PyType, PyTypeMethods};
#[cfg(not(PyPy))]
pub use self::weakref::{PyWeakCallableProxy, PyWeakProxy, PyWeakRef, PyWeakRefMethods};

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
/// Python::with_gil(|py| {
///     let dict = py.eval_bound("{'a':'b', 'c':'d'}", None, None)?.downcast_into::<PyDict>()?;
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
    pub use super::dict::{BoundDictIterator, PyDictIterator};
    pub use super::frozenset::{BoundFrozenSetIterator, PyFrozenSetIterator};
    pub use super::list::{BoundListIterator, PyListIterator};
    pub use super::set::{BoundSetIterator, PySetIterator};
    pub use super::tuple::{BorrowedTupleIterator, BoundTupleIterator, PyTupleIterator};
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

// Implementations core to all native types
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_base(
    ($name:ty $(;$generics:ident)* ) => {
        unsafe impl<$($generics,)*> $crate::PyNativeType for $name {
            type AsRefSource = Self;
        }

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
                use $crate::PyNativeType;
                match self.str() {
                    ::std::result::Result::Ok(s) => return f.write_str(&s.to_string_lossy()),
                    ::std::result::Result::Err(err) => err.write_unraisable_bound(self.py(), ::std::option::Option::Some(&self.as_borrowed())),
                }

                match self.get_type().name() {
                    ::std::result::Result::Ok(name) => ::std::write!(f, "<unprintable {} object>", name),
                    ::std::result::Result::Err(_err) => f.write_str("<unprintable object>"),
                }
            }
        }

        impl<$($generics,)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python<'_>) -> $crate::PyObject {
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

        unsafe impl<$($generics,)*> $crate::AsPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        impl<$($generics,)*> $crate::IntoPy<$crate::Py<$name>> for &'_ $name {
            #[inline]
            fn into_py(self, py: $crate::Python<'_>) -> $crate::Py<$name> {
                unsafe { $crate::Py::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        impl<$($generics,)*> ::std::convert::From<&'_ $name> for $crate::Py<$name> {
            #[inline]
            fn from(other: &$name) -> Self {
                use $crate::PyNativeType;
                unsafe { $crate::Py::from_borrowed_ptr(other.py(), other.as_ptr()) }
            }
        }

        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        impl<'a, $($generics,)*> ::std::convert::From<&'a $name> for &'a $crate::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::PyAny)}
            }
        }

        impl $crate::types::DerefToPyAny for $name {}
    };
);

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_static_type_object(
    ($typeobject:expr) => {
        |_py| unsafe { ::std::ptr::addr_of_mut!($typeobject) }
    };
);

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_info(
    ($name:ty, $typeobject:expr, $module:expr $(, #checkfunction=$checkfunction:path)? $(;$generics:ident)*) => {
        unsafe impl<$($generics,)*> $crate::type_object::PyTypeInfo for $name {
            const NAME: &'static str = stringify!($name);
            const MODULE: ::std::option::Option<&'static str> = $module;

            #[inline]
            #[allow(clippy::redundant_closure_call)]
            fn type_object_raw(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                $typeobject(py)
            }

            $(
                #[inline]
                fn is_type_of_bound(obj: &$crate::Bound<'_, $crate::PyAny>) -> bool {
                    #[allow(unused_unsafe)]
                    unsafe { $checkfunction(obj.as_ptr()) > 0 }
                }
            )?
        }

        impl<$($generics,)*> $crate::impl_::pymodule::PyAddToModule for $name {
            fn add_to_module(
                module: &$crate::Bound<'_, $crate::types::PyModule>,
            ) -> $crate::PyResult<()> {
                use $crate::types::PyModuleMethods;
                module.add(
                    <Self as $crate::PyTypeInfo>::NAME,
                    <Self as $crate::PyTypeInfo>::type_object_bound(module.py()),
                )
            }
        }
    };
);

// NOTE: This macro is not included in pyobject_native_type_base!
// because rust-numpy has a special implementation.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_extract {
    ($name:ty $(;$generics:ident)*) => {
        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        impl<'py, $($generics,)*> $crate::FromPyObject<'py> for &'py $name {
            #[inline]
            fn extract_bound(obj: &$crate::Bound<'py, $crate::PyAny>) -> $crate::PyResult<Self> {
                ::std::clone::Clone::clone(obj).into_gil_ref().downcast().map_err(::std::convert::Into::into)
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
            type LayoutAsBase = $crate::impl_::pycell::PyClassObjectBase<$layout>;
            type BaseNativeType = $name;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
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

pub(crate) mod any;
pub(crate) mod boolobject;
pub(crate) mod bytearray;
pub(crate) mod bytes;
pub(crate) mod capsule;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
mod code;
pub(crate) mod complex;
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod datetime;
pub(crate) mod dict;
mod ellipsis;
pub(crate) mod float;
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
mod frame;
pub(crate) mod frozenset;
mod function;
pub(crate) mod iterator;
pub(crate) mod list;
pub(crate) mod mapping;
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
#[cfg(not(PyPy))]
pub(crate) mod weakref;
