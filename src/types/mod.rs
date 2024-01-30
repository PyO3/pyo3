//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

pub use self::any::PyAny;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
pub use self::capsule::PyCapsule;
#[cfg(not(Py_LIMITED_API))]
pub use self::code::PyCode;
pub use self::complex::PyComplex;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::{
    timezone_utc, PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess,
    PyTzInfo, PyTzInfoAccess,
};
pub use self::dict::{IntoPyDict, PyDict};
#[cfg(not(PyPy))]
pub use self::dict::{PyDictItems, PyDictKeys, PyDictValues};
pub use self::ellipsis::PyEllipsis;
pub use self::float::PyFloat;
#[cfg(all(not(Py_LIMITED_API), not(PyPy)))]
pub use self::frame::PyFrame;
pub use self::frozenset::{PyFrozenSet, PyFrozenSetBuilder};
pub use self::function::PyCFunction;
#[cfg(all(not(Py_LIMITED_API), not(PyPy)))]
pub use self::function::PyFunction;
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::mapping::PyMapping;
pub use self::memoryview::PyMemoryView;
pub use self::module::PyModule;
pub use self::none::PyNone;
pub use self::notimplemented::PyNotImplemented;
pub use self::num::PyLong;
pub use self::num::PyLong as PyInt;
#[cfg(not(PyPy))]
pub use self::pysuper::PySuper;
pub use self::sequence::PySequence;
pub use self::set::PySet;
pub use self::slice::{PySlice, PySliceIndices};
#[cfg(not(Py_LIMITED_API))]
pub use self::string::PyStringData;
pub use self::string::{PyString, PyString as PyUnicode};
pub use self::traceback::PyTraceback;
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;

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
///     let dict: &PyDict = py.eval("{'a':'b', 'c':'d'}", None, None)?.downcast()?;
///
///     for (key, value) in dict {
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

        impl<$($generics,)*> $crate::IntoPy<$crate::Py<$name>> for &'_ $name {
            #[inline]
            fn into_py(self, py: $crate::Python<'_>) -> $crate::Py<$name> {
                unsafe { $crate::Py::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$($generics,)*> ::std::convert::From<&'_ $name> for $crate::Py<$name> {
            #[inline]
            fn from(other: &$name) -> Self {
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
                fn is_type_of(ptr: &$crate::PyAny) -> bool {
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
            #[inline]
            fn extract(obj: &'py $crate::PyAny) -> $crate::PyResult<Self> {
                obj.downcast().map_err(::std::convert::Into::into)
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
#[cfg(not(Py_LIMITED_API))]
mod code;
mod complex;
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod datetime;
pub(crate) mod dict;
mod ellipsis;
pub(crate) mod float;
#[cfg(all(not(Py_LIMITED_API), not(PyPy)))]
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
#[cfg(not(PyPy))]
mod pysuper;
pub(crate) mod sequence;
pub(crate) mod set;
mod slice;
pub(crate) mod string;
pub(crate) mod traceback;
pub(crate) mod tuple;
mod typeobject;
