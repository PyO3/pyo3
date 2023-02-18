pub use self::any::PyAny;
pub use self::dict::{IntoPyDict, PyDict};
pub use self::list::PyList;
pub use self::tuple::PyTuple;

// Implementations core to all native types
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_base_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty) => {
        unsafe impl<$py $(,$generics)*> $crate::experimental::PyUncheckedDowncast<$py> for $name {
            unsafe fn unchecked_downcast<'a>(obj: &'a $crate::experimental::PyAny<$py>) -> &'a Self {
                use $crate::AsPyPointer;
                let out: &'a Self = std::mem::transmute(obj);
                debug_assert_eq!(obj.as_ptr(), out.as_ptr());
                debug_assert_eq!(std::mem::size_of::<$crate::experimental::PyAny<'_>>(), std::mem::size_of::<Self>());
                out
            }
        }

        impl<$py $(,$generics)*> ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                   -> ::std::result::Result<(), ::std::fmt::Error>
            {
                let s = self.repr().or(::std::result::Result::Err(::std::fmt::Error))?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$py $(,$generics)*> ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                   -> ::std::result::Result<(), ::std::fmt::Error>
            {
                let s = self.str().or(::std::result::Result::Err(::std::fmt::Error))?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$py $(,$generics)*> $crate::ToPyObject for $name
        {
            #[inline]
            fn to_object(&self, py: $crate::Python<'_>) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }
    };
}

// Implementations core to all native types except for PyAny (because they don't
// make sense on PyAny / have different implementations).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_named_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty) => {
        $crate::pyobject_native_type_base_experimental!(impl<$py $(,$generics)*> $name);

        impl<$py $(,$generics)*> ::std::convert::AsRef<$crate::experimental::PyAny<$py>> for $name {
            #[inline]
            fn as_ref(&self) -> &$crate::experimental::PyAny<$py> {
                &self.0
            }
        }

        impl<$py $(,$generics)*> ::std::ops::Deref for $name {
            type Target = $crate::experimental::PyAny<$py>;

            #[inline]
            fn deref(&self) -> &$crate::experimental::PyAny<$py> {
                &self.0
            }
        }

        impl<$py $(,$generics)*> $crate::AsPyPointer for $name {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$py $(,$generics)*> $crate::IntoPy<$crate::Py<$name>> for &'_ $name {
            #[inline]
            fn into_py(self, py: $crate::Python<'_>) -> $crate::Py<$name> {
                use $crate::AsPyPointer;
                unsafe { $crate::Py::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$py $(,$generics)*> ::std::convert::From<&'_ $name> for $crate::Py<$name> {
            #[inline]
            fn from(other: &$name) -> Self {
                use $crate::AsPyPointer;
                unsafe { $crate::Py::from_borrowed_ptr(other.py(), other.as_ptr()) }
            }
        }

        impl<'a, $py $(,$generics)*> ::std::convert::From<&'a $name> for &'a $crate::PyAny {
            fn from(ob: &'a $name) -> Self {
                unsafe{&*(ob as *const $name as *const $crate::PyAny)}
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_info_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty, $typeobject:expr, $module:expr $(, #checkfunction=$checkfunction:path)?) => {
        unsafe impl<$py $(,$generics)*> $crate::type_object::PyTypeInfo for $name {
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
}

// NOTE: This macro is not included in pyobject_native_type_base!
// because rust-numpy has a special implementation.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_extract_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty) => {
        impl<$py $(,$generics)*> $crate::FromPyObject<$py> for $name {
            fn extract(obj: &'py $crate::PyAny) -> $crate::PyResult<Self> {
                let reference: &Self = $crate::experimental::PyAny::from_gil_ref(&obj).downcast()?;
                Ok(reference.clone())
            }
        }
    }
}

/// Declares all of the boilerplate for Python types.
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_core_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty, $typeobject:expr, #module=$module:expr $(, #checkfunction=$checkfunction:path)?) => {
        $crate::pyobject_native_type_named_experimental!(impl<$py $(,$generics)*> $name);
        $crate::pyobject_native_type_info_experimental!(impl<$py $(,$generics)*> $name, $typeobject, $module $(, #checkfunction=$checkfunction)?);
        $crate::pyobject_native_type_extract_experimental!(impl<$py $(,$generics)*> $name);
    };
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty, $typeobject:expr $(, #checkfunction=$checkfunction:path)?) => {
        $crate::pyobject_native_type_core_experimental!(impl<$py $(,$generics)*> $name, $typeobject, #module=::std::option::Option::Some("builtins") $(, #checkfunction=$checkfunction)?);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_sized_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty, $layout:path) => {
        unsafe impl<$py $(,$generics)*> $crate::type_object::PyLayout<$name> for $layout {}
        impl<$py $(,$generics)*> $crate::type_object::PySizedLayout<$name> for $layout {}
        impl<$py $(,$generics)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::pycell::PyCellBase<$layout>;
            type BaseNativeType = $name;
            type ThreadChecker = $crate::impl_::pyclass::NativeTypeThreadCheckerStub;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
        }
    }
}

/// Declares all of the boilerplate for Python types which can be inherited from (because the exact
/// Python layout is known).
#[doc(hidden)]
#[macro_export]
macro_rules! pyobject_native_type_experimental {
    (impl<$py:lifetime $(,$generics:ident)* $(,)?> $name:ty, $layout:path, $typeobject:expr $(, #module=$module:expr)? $(, #checkfunction=$checkfunction:path)?) => {
        $crate::pyobject_native_type_core_experimental!(impl<$py $(,$generics)*> $name, $typeobject $(, #module=$module)? $(, #checkfunction=$checkfunction)?);
        // To prevent inheriting native types with ABI3
        #[cfg(not(Py_LIMITED_API))]
        $crate::pyobject_native_type_sized_experimental!(impl<$py $(,$generics)*> $name, $layout);
    };
}

mod any;
pub mod detached;
mod dict;
mod list;
mod tuple;
