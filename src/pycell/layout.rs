#![allow(missing_docs)]
//! Crate-private implementation of how PyClassObjects are laid out in memory and how to access data from raw PyObjects

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::addr_of_mut;

use memoffset::offset_of;

use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef, PyObjectOffset,
};
use crate::internal::get_slot::{TP_DEALLOC, TP_FREE};
use crate::internal_tricks::{cast_const, cast_mut};
use crate::pycell::borrow_checker::{GetBorrowChecker, PyClassBorrowChecker};
use crate::type_object::PyNativeType;
use crate::types::PyType;
use crate::{ffi, PyTypeInfo, Python};

#[cfg(not(Py_LIMITED_API))]
use crate::types::PyTypeMethods;

use super::borrow_checker::PyClassMutability;
use super::{ptr_from_ref, PyBorrowError};

/// The layout of the region of a [ffi::PyObject] specifically relating to type `T`.
///
/// In an inheritance hierarchy where `#[pyclass(extends=PyDict)] struct A;` and `#[pyclass(extends=A)] struct B;`
/// a [ffi::PyObject] of type `B` has separate memory for [ffi::PyDictObject] (the base native type) and
/// `PyClassObjectContents<A>` and `PyClassObjectContents<B>`. The memory associated with `A` or `B` can be obtained
/// using `PyObjectLayout::get_contents::<T>()` (where `T=A` or `T=B`).
#[repr(C)]
pub(crate) struct PyClassObjectContents<T: PyClassImpl> {
    /// The data associated with the user-defined struct annotated with `#[pyclass]`
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    /// A pointer to a [ffi::PyObject] if `T` is annotated with `#[pyclass(dict)]` and a zero-sized field otherwise.
    pub(crate) dict: T::Dict,
    /// A pointer to a [ffi::PyObject] if `T` is annotated with `#[pyclass(weakref)]` and a zero-sized field otherwise.
    pub(crate) weakref: T::WeakRef,
}

impl<T: PyClassImpl> PyClassObjectContents<T> {
    pub(crate) fn new(init: T) -> Self {
        PyClassObjectContents {
            value: ManuallyDrop::new(UnsafeCell::new(init)),
            borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage::new(),
            thread_checker: T::ThreadChecker::new(),
            dict: T::Dict::INIT,
            weakref: T::WeakRef::INIT,
        }
    }

    unsafe fn dealloc(&mut self, py: Python<'_>, py_object: *mut ffi::PyObject) {
        if self.thread_checker.can_drop(py) {
            ManuallyDrop::drop(&mut self.value);
        }
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(py_object, py);
    }
}

/// Functions for working with [ffi::PyObject]s recursively by re-interpreting the object
/// as being an instance of the most derived class through each base class until
/// the `BaseNativeType` is reached.
///
/// E.g. if `#[pyclass(extends=PyDict)] struct A;` and `#[pyclass(extends=A)] struct B;`
/// then calling a method on a PyObject of type `B` will call the method for `B`, then `A`, then `PyDict`.
#[doc(hidden)]
pub trait PyObjectRecursiveOperations {
    /// [PyTypeInfo::type_object_raw()] may create type objects lazily.
    /// This method ensures that the type objects for all ancestor types of the provided object.
    fn ensure_type_objects_initialized(py: Python<'_>);

    /// Call [PyClassThreadChecker::ensure()] on all ancestor types of the provided object.
    fn ensure_threadsafe(obj: &ffi::PyObject, strategy: TypeObjectStrategy<'_>);

    /// Call [PyClassThreadChecker::check()] on all ancestor types of the provided object.
    fn check_threadsafe(
        obj: &ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> Result<(), PyBorrowError>;

    /// Cleanup then free the memory for `obj`.
    ///
    /// # Safety
    /// - `obj` must be a valid pointer to an instance of a `T` or a subclass.
    /// - `obj` must not be used after this call (as it will be freed).
    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject);
}

/// Used to fill out [PyClassBaseType::RecursiveOperations] for instances of `PyClass`
pub struct PyClassRecursiveOperations<T>(PhantomData<T>);

impl<T: PyClassImpl + PyTypeInfo> PyObjectRecursiveOperations for PyClassRecursiveOperations<T> {
    fn ensure_type_objects_initialized(py: Python<'_>) {
        let _ = <T as PyTypeInfo>::type_object_raw(py);
        <T::BaseType as PyClassBaseType>::RecursiveOperations::ensure_type_objects_initialized(py);
    }

    fn ensure_threadsafe(obj: &ffi::PyObject, strategy: TypeObjectStrategy<'_>) {
        let contents = unsafe { PyObjectLayout::get_contents::<T>(obj, strategy) };
        contents.thread_checker.ensure();
        <T::BaseType as PyClassBaseType>::RecursiveOperations::ensure_threadsafe(obj, strategy);
    }

    fn check_threadsafe(
        obj: &ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> Result<(), PyBorrowError> {
        let contents = unsafe { PyObjectLayout::get_contents::<T>(obj, strategy) };
        if !contents.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        <T::BaseType as PyClassBaseType>::RecursiveOperations::check_threadsafe(obj, strategy)
    }

    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let contents =
            &mut *PyObjectLayout::get_contents_ptr::<T>(obj, TypeObjectStrategy::lazy(py));
        contents.dealloc(py, obj);
        <T::BaseType as PyClassBaseType>::RecursiveOperations::deallocate(py, obj);
    }
}

/// Used to fill out [PyClassBaseType::RecursiveOperations] for native types
pub struct PyNativeTypeRecursiveOperations<T>(PhantomData<T>);

impl<T: PyNativeType + PyTypeInfo> PyObjectRecursiveOperations
    for PyNativeTypeRecursiveOperations<T>
{
    fn ensure_type_objects_initialized(py: Python<'_>) {
        let _ = <T as PyTypeInfo>::type_object_raw(py);
    }

    fn ensure_threadsafe(_obj: &ffi::PyObject, _strategy: TypeObjectStrategy<'_>) {}

    fn check_threadsafe(
        _obj: &ffi::PyObject,
        _strategy: TypeObjectStrategy<'_>,
    ) -> Result<(), PyBorrowError> {
        Ok(())
    }

    /// Call the destructor (`tp_dealloc`) of an object which is an instance of a
    /// subclass of the native type `T`.
    ///
    /// Does not clear up any data from subtypes of `type_ptr` so it is assumed that those
    /// destructors have been called first.
    ///
    /// [tp_dealloc docs](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dealloc)
    ///
    /// # Safety
    /// - `obj` must be a valid pointer to an instance of the type at `type_ptr` or a subclass.
    /// - `obj` must not be used after this call (as it will be freed).
    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject) {
        // the `BaseNativeType` of the object
        let type_ptr = <T as PyTypeInfo>::type_object_raw(py);

        // FIXME: there is potentially subtle issues here if the base is overwritten at runtime? To be investigated.

        // the 'most derived class' of `obj`. i.e. the result of calling `type(obj)`.
        let actual_type = PyType::from_borrowed_type_ptr(py, ffi::Py_TYPE(obj));

        if type_ptr == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type) {
            // the `PyBaseObject_Type` destructor (tp_dealloc) just calls tp_free so we can do this directly
            let tp_free = actual_type
                .get_slot(TP_FREE)
                .expect("base type should have tp_free");
            return tp_free(obj.cast());
        }

        if type_ptr == std::ptr::addr_of_mut!(ffi::PyType_Type) {
            let tp_dealloc = PyType::from_borrowed_type_ptr(py, type_ptr)
                .get_slot(TP_DEALLOC)
                .expect("PyType_Type should have tp_dealloc");
            // `PyType_Type::dealloc` calls `Py_GC_UNTRACK` so we have to re-track before deallocating
            #[cfg(not(PyPy))]
            ffi::PyObject_GC_Track(obj.cast());
            return tp_dealloc(obj.cast());
        }

        // More complex native types (e.g. `extends=PyDict`) require calling the base's dealloc.
        #[cfg(not(Py_LIMITED_API))]
        {
            // FIXME: should this be using actual_type.tp_dealloc?
            if let Some(dealloc) = (*type_ptr).tp_dealloc {
                // Before CPython 3.11 BaseException_dealloc would use Py_GC_UNTRACK which
                // assumes the exception is currently GC tracked, so we have to re-track
                // before calling the dealloc so that it can safely call Py_GC_UNTRACK.
                #[cfg(not(any(Py_3_11, PyPy)))]
                if ffi::PyType_FastSubclass(type_ptr, ffi::Py_TPFLAGS_BASE_EXC_SUBCLASS) == 1 {
                    ffi::PyObject_GC_Track(obj.cast());
                }
                dealloc(obj);
            } else {
                (*actual_type.as_type_ptr())
                    .tp_free
                    .expect("type missing tp_free")(obj.cast());
            }
        }

        #[cfg(Py_LIMITED_API)]
        unreachable!("subclassing native types is not possible with the `abi3` feature");
    }
}

/// Utilities for working with [ffi::PyObject] objects that utilise [PEP 697](https://peps.python.org/pep-0697/).
#[doc(hidden)]
pub(crate) mod opaque_layout {
    #[cfg(Py_3_12)]
    use super::{PyClassObjectContents, TypeObjectStrategy};
    #[cfg(Py_3_12)]
    use crate::ffi;
    use crate::{impl_::pyclass::PyClassImpl, PyTypeInfo};

    /// Obtain a pointer to the region of `obj` that relates to `T`
    ///
    /// # Safety
    /// - `obj` must be a valid `ffi::PyObject` of type `T` or a subclass of `T` that uses the opaque layout
    #[cfg(Py_3_12)]
    pub(crate) unsafe fn get_contents_ptr<T: PyClassImpl + PyTypeInfo>(
        obj: *mut ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> *mut PyClassObjectContents<T> {
        let type_obj = match strategy {
            TypeObjectStrategy::Lazy(py) => T::type_object_raw(py),
            TypeObjectStrategy::AssumeInit(_) => {
                T::try_get_type_object_raw().unwrap_or_else(|| {
                    panic!(
                        "type object for {} not initialized",
                        std::any::type_name::<T>()
                    )
                })
            }
        };
        assert!(!type_obj.is_null(), "type object is NULL");
        debug_assert!(
            unsafe { ffi::PyType_IsSubtype(ffi::Py_TYPE(obj), type_obj) } == 1,
            "the object is not an instance of {}",
            std::any::type_name::<T>()
        );
        let pointer = unsafe { ffi::PyObject_GetTypeData(obj, type_obj) };
        assert!(!pointer.is_null(), "pointer to pyclass data returned NULL");
        pointer.cast()
    }

    #[inline(always)]
    #[cfg(not(Py_3_12))]
    pub fn panic_unsupported<T: PyClassImpl + PyTypeInfo>() -> ! {
        assert!(T::OPAQUE);
        panic!(
            "The opaque object layout (used by {}) is not supported until python 3.12",
            std::any::type_name::<T>()
        );
    }
}

/// Utilities for working with [ffi::PyObject] objects that utilise the standard layout for python extensions,
/// where the base class is placed at the beginning of a `repr(C)` struct.
#[doc(hidden)]
pub(crate) mod static_layout {
    use crate::{
        impl_::pyclass::{PyClassBaseType, PyClassImpl},
        type_object::{PyLayout, PySizedLayout},
    };

    use super::PyClassObjectContents;

    // The layout of a [ffi::PyObject] that uses the static layout
    #[repr(C)]
    pub struct PyStaticClassLayout<T: PyClassImpl> {
        pub(crate) ob_base: <T::BaseType as PyClassBaseType>::StaticLayout,
        pub(crate) contents: PyClassObjectContents<T>,
    }

    unsafe impl<T: PyClassImpl> PyLayout<T> for PyStaticClassLayout<T> {}

    /// Layout of a native type `T` with a known size (not opaque)
    /// Corresponds to [PyObject](https://docs.python.org/3/c-api/structures.html#c.PyObject) from the C API.
    #[doc(hidden)]
    #[repr(C)]
    pub struct PyStaticNativeLayout<T> {
        ob_base: T,
    }

    unsafe impl<T, U> PyLayout<T> for PyStaticNativeLayout<U> where U: PySizedLayout<T> {}

    /// a struct for use with opaque native types to indicate that they
    /// cannot be used as part of a static layout.
    #[repr(C)]
    pub struct InvalidStaticLayout;

    /// This is valid insofar as casting a `*mut ffi::PyObject` to `*mut InvalidStaticLayout` is valid
    /// since `InvalidStaticLayout` has no fields to read.
    unsafe impl<T> PyLayout<T> for InvalidStaticLayout {}
}

/// The method to use for obtaining a [ffi::PyTypeObject] pointer describing `T: PyTypeInfo` for
/// use with [PyObjectLayout] functions.
///
/// [PyTypeInfo::type_object_raw()] requires the GIL to be held because it may lazily construct the type object.
/// Some situations require that the GIL is not held so [PyObjectLayout] cannot call this method directly.
/// The different solutions to this have different trade-offs.
#[derive(Clone, Copy)]
pub enum TypeObjectStrategy<'a> {
    Lazy(Python<'a>),
    AssumeInit(PhantomData<&'a ()>),
}

impl<'a> TypeObjectStrategy<'a> {
    /// Hold the GIL and only obtain/construct type objects lazily when required.
    pub fn lazy(py: Python<'a>) -> Self {
        TypeObjectStrategy::Lazy(py)
    }

    /// Assume that [PyTypeInfo::type_object_raw()] has been called for any of the required type objects.
    ///
    /// Once initialized, the type objects are cached and can be obtained without holding the GIL.
    ///
    /// # Safety
    ///
    /// - Ensure that any `T` that may be used with this strategy has already been initialized
    ///   by calling [PyTypeInfo::type_object_raw()].
    /// - Only [PyTypeInfo::OPAQUE] classes require type objects for traversal so if this strategy is only
    ///   used with non-opaque classes then no action is required.
    /// - When used with [PyClassRecursiveOperations] or [GetBorrowChecker], the strategy may be used with
    ///   base classes as well as the most derived type.
    ///   [PyClassRecursiveOperations::ensure_type_objects_initialized()] can be used to initialize
    ///   all base classes above the given type.
    pub unsafe fn assume_init() -> Self {
        TypeObjectStrategy::AssumeInit(PhantomData)
    }
}

/// Functions for working with [ffi::PyObject]s
pub(crate) struct PyObjectLayout {}

impl PyObjectLayout {
    /// Obtain a pointer to the portion of `obj` relating to the type `T`
    ///
    /// # Safety
    /// `obj` must point to a valid `PyObject` whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn get_contents_ptr<T: PyClassImpl + PyTypeInfo>(
        obj: *mut ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> *mut PyClassObjectContents<T> {
        debug_assert!(!obj.is_null(), "get_contents_ptr of null object");
        if T::OPAQUE {
            #[cfg(Py_3_12)]
            {
                opaque_layout::get_contents_ptr(obj, strategy)
            }

            #[cfg(not(Py_3_12))]
            {
                let _ = strategy;
                opaque_layout::panic_unsupported::<T>();
            }
        } else {
            let obj: *mut static_layout::PyStaticClassLayout<T> = obj.cast();
            // indicates `ob_base` has type [static_layout::InvalidStaticLayout]
            debug_assert_ne!(
                offset_of!(static_layout::PyStaticClassLayout<T>, contents),
                0,
                "invalid ob_base found"
            );
            addr_of_mut!((*obj).contents)
        }
    }

    /// Obtain a reference to the portion of `obj` relating to the type `T`
    ///
    /// # Safety
    /// `obj` must point to a valid `PyObject` whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn get_contents<'a, T: PyClassImpl + PyTypeInfo>(
        obj: &'a ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> &'a PyClassObjectContents<T> {
        &*cast_const(PyObjectLayout::get_contents_ptr::<T>(
            cast_mut(ptr_from_ref(obj)),
            strategy,
        ))
    }

    /// Obtain a pointer to the portion of `obj` containing the user data for `T`
    ///
    /// # Safety
    /// `obj` must point to a valid `PyObject` whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn get_data_ptr<T: PyClassImpl + PyTypeInfo>(
        obj: *mut ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> *mut T {
        let contents = PyObjectLayout::get_contents_ptr::<T>(obj, strategy);
        (*contents).value.get()
    }

    /// Obtain a reference to the portion of `obj` containing the user data for `T`
    ///
    /// # Safety
    /// `obj` must point to a valid [ffi::PyObject] whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn get_data<'a, T: PyClassImpl + PyTypeInfo>(
        obj: &'a ffi::PyObject,
        strategy: TypeObjectStrategy<'_>,
    ) -> &'a T {
        &*PyObjectLayout::get_data_ptr::<T>(cast_mut(ptr_from_ref(obj)), strategy)
    }

    /// Obtain a reference to the borrow checker for `obj`
    ///
    /// Note: this method is for convenience. The implementation is in [GetBorrowChecker].
    ///
    /// # Safety
    /// `obj` must point to a valid [ffi::PyObject] whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn get_borrow_checker<'a, T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: &'a ffi::PyObject,
    ) -> &'a <T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(obj, TypeObjectStrategy::lazy(py))
    }

    /// Ensure that `obj` is thread safe.
    ///
    /// Note: this method is for convenience. The implementation is in [PyClassRecursiveOperations].
    ///
    /// # Safety
    /// `obj` must point to a valid [ffi::PyObject] whose type is `T` or a subclass of `T`.
    pub(crate) unsafe fn ensure_threadsafe<T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: &ffi::PyObject,
    ) {
        PyClassRecursiveOperations::<T>::ensure_threadsafe(obj, TypeObjectStrategy::lazy(py));
    }

    /// Clean up then free the memory associated with `obj`.
    ///
    /// Note: this method is for convenience. The implementation is in [PyClassRecursiveOperations].
    ///
    /// See [tp_dealloc docs](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dealloc)
    pub(crate) unsafe fn deallocate<T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: *mut ffi::PyObject,
    ) {
        PyClassRecursiveOperations::<T>::deallocate(py, obj);
    }

    /// Clean up then free the memory associated with `obj`.
    ///
    /// Use instead of `deallocate()` if `T` has the `Py_TPFLAGS_HAVE_GC` flag set.
    ///
    /// See [tp_dealloc docs](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dealloc)
    pub(crate) unsafe fn deallocate_with_gc<T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: *mut ffi::PyObject,
    ) {
        #[cfg(not(PyPy))]
        {
            ffi::PyObject_GC_UnTrack(obj.cast());
        }
        PyClassRecursiveOperations::<T>::deallocate(py, obj);
    }

    /// Used to set `PyType_Spec::basicsize` when creating a `PyTypeObject` for `T`
    /// ([docs](https://docs.python.org/3/c-api/type.html#c.PyType_Spec.basicsize))
    pub(crate) fn basicsize<T: PyClassImpl + PyTypeInfo>() -> ffi::Py_ssize_t {
        if T::OPAQUE {
            #[cfg(Py_3_12)]
            {
                // negative to indicate 'extra' space that python will allocate
                // specifically for `T` excluding the base class.
                -usize_to_py_ssize(std::mem::size_of::<PyClassObjectContents<T>>())
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported::<T>();
        } else {
            usize_to_py_ssize(std::mem::size_of::<static_layout::PyStaticClassLayout<T>>())
        }
    }

    /// Gets the offset of the contents from the start of the struct in bytes.
    pub(crate) fn contents_offset<T: PyClassImpl + PyTypeInfo>() -> PyObjectOffset {
        if T::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(0)
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported::<T>();
        } else {
            PyObjectOffset::Absolute(usize_to_py_ssize(memoffset::offset_of!(
                static_layout::PyStaticClassLayout<T>,
                contents
            )))
        }
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    pub(crate) fn dict_offset<T: PyClassImpl + PyTypeInfo>() -> PyObjectOffset {
        if T::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
                    PyClassObjectContents<T>,
                    dict
                )))
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported::<T>();
        } else {
            let offset = memoffset::offset_of!(static_layout::PyStaticClassLayout<T>, contents)
                + memoffset::offset_of!(PyClassObjectContents<T>, dict);

            PyObjectOffset::Absolute(usize_to_py_ssize(offset))
        }
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    pub(crate) fn weaklist_offset<T: PyClassImpl + PyTypeInfo>() -> PyObjectOffset {
        if T::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
                    PyClassObjectContents<T>,
                    weakref
                )))
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported::<T>();
        } else {
            let offset = memoffset::offset_of!(static_layout::PyStaticClassLayout<T>, contents)
                + memoffset::offset_of!(PyClassObjectContents<T>, weakref);

            PyObjectOffset::Absolute(usize_to_py_ssize(offset))
        }
    }
}

/// Py_ssize_t may not be equal to isize on all platforms
pub(crate) fn usize_to_py_ssize(value: usize) -> ffi::Py_ssize_t {
    #[allow(clippy::useless_conversion)]
    value.try_into().expect("value should fit in Py_ssize_t")
}

/// Tests specific to the static layout
#[cfg(all(test, feature = "macros"))]
#[allow(clippy::bool_comparison)] // `== false` is harder to miss than !
mod static_tests {
    use static_assertions::const_assert;

    #[cfg(not(Py_LIMITED_API))]
    use super::test_utils::get_pyobject_size;
    use super::*;

    use crate::prelude::*;
    use memoffset::offset_of;
    use std::mem::size_of;

    /// Test the functions calculate properties about the static layout without requiring an instance.
    /// The class in this test extends the default base class `PyAny` so there is 'no inheritance'.
    #[test]
    fn test_type_properties_no_inheritance() {
        #[pyclass(crate = "crate", extends=PyAny)]
        struct MyClass(#[allow(unused)] u64);

        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == false);

        #[repr(C)]
        struct ExpectedLayout {
            /// typically called `ob_base`. In C it is defined using the `PyObject_HEAD` macro
            /// [docs](https://docs.python.org/3/c-api/structures.html)
            native_base: ffi::PyObject,
            contents: PyClassObjectContents<MyClass>,
        }

        let expected_size = size_of::<ExpectedLayout>() as ffi::Py_ssize_t;
        assert_eq!(PyObjectLayout::basicsize::<MyClass>(), expected_size);

        let expected_contents_offset = offset_of!(ExpectedLayout, contents) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::contents_offset::<MyClass>(),
            PyObjectOffset::Absolute(expected_contents_offset),
        );

        let dict_size = size_of::<<MyClass as PyClassImpl>::Dict>();
        assert_eq!(dict_size, 0);
        let expected_dict_offset_in_contents =
            offset_of!(PyClassObjectContents<MyClass>, dict) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::dict_offset::<MyClass>(),
            PyObjectOffset::Absolute(expected_contents_offset + expected_dict_offset_in_contents),
        );

        let weakref_size = size_of::<<MyClass as PyClassImpl>::WeakRef>();
        assert_eq!(weakref_size, 0);
        let expected_weakref_offset_in_contents =
            offset_of!(PyClassObjectContents<MyClass>, weakref) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::weaklist_offset::<MyClass>(),
            PyObjectOffset::Absolute(
                expected_contents_offset + expected_weakref_offset_in_contents
            ),
        );

        assert_eq!(
            expected_dict_offset_in_contents,
            expected_weakref_offset_in_contents
        );
    }

    /// Test the functions calculate properties about the static layout without requiring an instance.
    /// The class in this test requires extra space for the `dict` and `weaklist` fields
    #[test]
    #[cfg(any(Py_3_9, not(Py_LIMITED_API)))]
    fn test_layout_properties_no_inheritance_optional_fields() {
        #[pyclass(crate = "crate", dict, weakref, extends=PyAny)]
        struct MyClass(#[allow(unused)] u64);

        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == false);

        #[repr(C)]
        struct ExpectedLayout {
            native_base: ffi::PyObject,
            contents: PyClassObjectContents<MyClass>,
        }

        let expected_size = size_of::<ExpectedLayout>() as ffi::Py_ssize_t;
        assert_eq!(PyObjectLayout::basicsize::<MyClass>(), expected_size);

        let expected_contents_offset = offset_of!(ExpectedLayout, contents) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::contents_offset::<MyClass>(),
            PyObjectOffset::Absolute(expected_contents_offset),
        );

        let dict_size = size_of::<<MyClass as PyClassImpl>::Dict>();
        assert!(dict_size > 0);
        let expected_dict_offset_in_contents =
            offset_of!(PyClassObjectContents<MyClass>, dict) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::dict_offset::<MyClass>(),
            PyObjectOffset::Absolute(expected_contents_offset + expected_dict_offset_in_contents),
        );

        let weakref_size = size_of::<<MyClass as PyClassImpl>::WeakRef>();
        assert!(weakref_size > 0);
        let expected_weakref_offset_in_contents =
            offset_of!(PyClassObjectContents<MyClass>, weakref) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::weaklist_offset::<MyClass>(),
            PyObjectOffset::Absolute(
                expected_contents_offset + expected_weakref_offset_in_contents
            ),
        );

        assert!(expected_dict_offset_in_contents < expected_weakref_offset_in_contents);
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_type_properties_with_inheritance() {
        use std::any::TypeId;

        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, extends=PyDict)]
        struct ParentClass {
            #[allow(unused)]
            parent_field: u64,
        }

        #[pyclass(crate = "crate", extends=ParentClass)]
        struct ChildClass {
            #[allow(unused)]
            child_field: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == false);
        assert_eq!(
            TypeId::of::<<ChildClass as PyClassImpl>::BaseType>(),
            TypeId::of::<ParentClass>()
        );
        assert_eq!(
            TypeId::of::<<ParentClass as PyClassImpl>::BaseType>(),
            TypeId::of::<PyDict>()
        );

        #[repr(C)]
        struct ExpectedLayout {
            native_base: ffi::PyDictObject,
            parent_contents: PyClassObjectContents<ParentClass>,
            child_contents: PyClassObjectContents<ChildClass>,
        }

        let expected_size = size_of::<ExpectedLayout>() as ffi::Py_ssize_t;
        assert_eq!(PyObjectLayout::basicsize::<ChildClass>(), expected_size);

        Python::with_gil(|py| {
            let typ_size = get_pyobject_size::<ChildClass>(py) as isize;
            assert_eq!(typ_size, expected_size);
        });

        let expected_parent_contents_offset =
            offset_of!(ExpectedLayout, parent_contents) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::contents_offset::<ParentClass>(),
            PyObjectOffset::Absolute(expected_parent_contents_offset),
        );

        let expected_child_contents_offset =
            offset_of!(ExpectedLayout, child_contents) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::contents_offset::<ChildClass>(),
            PyObjectOffset::Absolute(expected_child_contents_offset),
        );

        let child_dict_size = size_of::<<ChildClass as PyClassImpl>::Dict>();
        assert_eq!(child_dict_size, 0);
        let expected_child_dict_offset_in_contents =
            offset_of!(PyClassObjectContents<ChildClass>, dict) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::dict_offset::<ChildClass>(),
            PyObjectOffset::Absolute(
                expected_child_contents_offset + expected_child_dict_offset_in_contents
            ),
        );

        let child_weakref_size = size_of::<<ChildClass as PyClassImpl>::WeakRef>();
        assert_eq!(child_weakref_size, 0);
        let expected_child_weakref_offset_in_contents =
            offset_of!(PyClassObjectContents<ChildClass>, weakref) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::weaklist_offset::<ChildClass>(),
            PyObjectOffset::Absolute(
                expected_child_contents_offset + expected_child_weakref_offset_in_contents
            ),
        );
    }

    /// Test the functions that operate on pyclass instances
    /// The class in this test extends the default base class `PyAny` so there is 'no inheritance'.
    #[test]
    fn test_contents_access_no_inheritance() {
        #[pyclass(crate = "crate", extends=PyAny)]
        struct MyClass {
            my_value: u64,
        }

        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == false);

        #[repr(C)]
        struct ExpectedLayout {
            native_base: ffi::PyObject,
            contents: PyClassObjectContents<MyClass>,
        }

        Python::with_gil(|py| {
            let obj = Py::new(py, MyClass { my_value: 123 }).unwrap();
            let obj_ptr = obj.as_ptr();

            // test obtaining contents pointer normally (with GIL held)
            let contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<MyClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };

            // work around the fact that pointers are not `Send`
            let obj_ptr_int = obj_ptr as usize;
            let contents_ptr_int = contents_ptr as usize;

            // test that the contents pointer can be obtained without the GIL held
            py.allow_threads(move || {
                let obj_ptr = obj_ptr_int as *mut ffi::PyObject;
                let contents_ptr = contents_ptr_int as *mut PyClassObjectContents<MyClass>;

                // Safety: type object was created when `obj` was constructed
                let contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<MyClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                assert_eq!(contents_ptr, contents_ptr_without_gil);
            });

            // test that contents pointer matches expecations
            let casted_obj = obj_ptr.cast::<ExpectedLayout>();
            let expected_contents_ptr = unsafe { addr_of_mut!((*casted_obj).contents) };
            assert_eq!(contents_ptr, expected_contents_ptr);

            // test getting contents by reference
            let contents = unsafe {
                PyObjectLayout::get_contents::<MyClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(contents), expected_contents_ptr);

            // test getting data pointer
            let data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<MyClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let expected_data_ptr = unsafe { (*expected_contents_ptr).value.get() };
            assert_eq!(data_ptr, expected_data_ptr);
            assert_eq!(unsafe { (*data_ptr).my_value }, 123);

            // test getting data by reference
            let data = unsafe {
                PyObjectLayout::get_data::<MyClass>(obj.as_raw_ref(), TypeObjectStrategy::lazy(py))
            };
            assert_eq!(ptr_from_ref(data), expected_data_ptr);
        });
    }

    /// Test the functions that operate on pyclass instances.
    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_contents_access_with_inheritance() {
        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, extends=PyDict)]
        struct ParentClass {
            parent_value: u64,
        }

        #[pyclass(crate = "crate", extends=ParentClass)]
        struct ChildClass {
            child_value: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == false);

        #[repr(C)]
        struct ExpectedLayout {
            native_base: ffi::PyDictObject,
            parent_contents: PyClassObjectContents<ParentClass>,
            child_contents: PyClassObjectContents<ChildClass>,
        }

        Python::with_gil(|py| {
            let obj = Py::new(
                py,
                PyClassInitializer::from(ParentClass { parent_value: 123 }).add_subclass(
                    ChildClass {
                        child_value: "foo".to_owned(),
                    },
                ),
            )
            .unwrap();
            let obj_ptr = obj.as_ptr();

            // test obtaining contents pointer normally (with GIL held)
            let parent_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ParentClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ChildClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };

            // work around the fact that pointers are not `Send`
            let obj_ptr_int = obj_ptr as usize;
            let parent_contents_ptr_int = parent_contents_ptr as usize;
            let child_contents_ptr_int = child_contents_ptr as usize;

            // test that the contents pointer can be obtained without the GIL held
            py.allow_threads(move || {
                let obj_ptr = obj_ptr_int as *mut ffi::PyObject;
                let parent_contents_ptr =
                    parent_contents_ptr_int as *mut PyClassObjectContents<ParentClass>;
                let child_contents_ptr =
                    child_contents_ptr_int as *mut PyClassObjectContents<ChildClass>;

                // Safety: type object was created when `obj` was constructed
                let parent_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ParentClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                let child_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ChildClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                assert_eq!(parent_contents_ptr, parent_contents_ptr_without_gil);
                assert_eq!(child_contents_ptr, child_contents_ptr_without_gil);
            });

            // test that contents pointer matches expecations
            let casted_obj = obj_ptr.cast::<ExpectedLayout>();
            let expected_parent_contents_ptr =
                unsafe { addr_of_mut!((*casted_obj).parent_contents) };
            let expected_child_contents_ptr = unsafe { addr_of_mut!((*casted_obj).child_contents) };
            assert_eq!(parent_contents_ptr, expected_parent_contents_ptr);
            assert_eq!(child_contents_ptr, expected_child_contents_ptr);

            // test getting contents by reference
            let parent_contents = unsafe {
                PyObjectLayout::get_contents::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents = unsafe {
                PyObjectLayout::get_contents::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_contents), expected_parent_contents_ptr);
            assert_eq!(ptr_from_ref(child_contents), expected_child_contents_ptr);

            // test getting data pointer
            let parent_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ParentClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let child_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ChildClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let expected_parent_data_ptr = unsafe { (*expected_parent_contents_ptr).value.get() };
            let expected_child_data_ptr = unsafe { (*expected_child_contents_ptr).value.get() };
            assert_eq!(parent_data_ptr, expected_parent_data_ptr);
            assert_eq!(unsafe { (*parent_data_ptr).parent_value }, 123);
            assert_eq!(child_data_ptr, expected_child_data_ptr);
            assert_eq!(unsafe { &(*child_data_ptr).child_value }, "foo");

            // test getting data by reference
            let parent_data = unsafe {
                PyObjectLayout::get_data::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_data = unsafe {
                PyObjectLayout::get_data::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_data), expected_parent_data_ptr);
            assert_eq!(ptr_from_ref(child_data), expected_child_data_ptr);
        });
    }

    #[test]
    fn test_inherited_size() {
        #[pyclass(crate = "crate", subclass)]
        struct ParentClass;

        #[pyclass(crate = "crate", extends = ParentClass)]
        struct ChildClass(#[allow(unused)] u64);

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == false);

        #[repr(C)]
        struct ExpectedLayoutWithData {
            native_base: ffi::PyObject,
            parent_class: PyClassObjectContents<ParentClass>,
            child_class: PyClassObjectContents<ChildClass>,
        }
        let expected_size = std::mem::size_of::<ExpectedLayoutWithData>() as ffi::Py_ssize_t;

        assert_eq!(PyObjectLayout::basicsize::<ChildClass>(), expected_size);
    }

    /// Test that `Drop::drop` is called for pyclasses
    #[test]
    fn test_destructor_called() {
        use std::sync::{Arc, Mutex};

        let deallocations: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        #[pyclass(crate = "crate", subclass)]
        struct ParentClass(Arc<Mutex<Vec<String>>>);

        impl Drop for ParentClass {
            fn drop(&mut self) {
                self.0.lock().unwrap().push("ParentClass".to_owned());
            }
        }

        #[pyclass(crate = "crate", extends = ParentClass)]
        struct ChildClass(Arc<Mutex<Vec<String>>>);

        impl Drop for ChildClass {
            fn drop(&mut self) {
                self.0.lock().unwrap().push("ChildClass".to_owned());
            }
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == false);

        Python::with_gil(|py| {
            let obj = Py::new(
                py,
                PyClassInitializer::from(ParentClass(deallocations.clone()))
                    .add_subclass(ChildClass(deallocations.clone())),
            )
            .unwrap();
            assert!(deallocations.lock().unwrap().is_empty());
            drop(obj);
        });

        assert_eq!(
            deallocations.lock().unwrap().as_slice(),
            &["ChildClass", "ParentClass"]
        );
    }

    #[test]
    fn test_empty_class() {
        #[pyclass(crate = "crate")]
        struct EmptyClass;

        // even if the user class has no data some additional space is required
        assert!(size_of::<PyClassObjectContents<EmptyClass>>() > 0);
    }

    /// It is essential that `InvalidStaticLayout` has 0 size so that it can be distinguished from a valid layout
    #[test]
    fn test_invalid_base() {
        assert_eq!(std::mem::size_of::<static_layout::InvalidStaticLayout>(), 0);

        #[repr(C)]
        struct InvalidLayout {
            ob_base: static_layout::InvalidStaticLayout,
            contents: u8,
        }

        assert_eq!(offset_of!(InvalidLayout, contents), 0);
    }
}

/// Tests specific to the opaque layout
#[cfg(all(test, Py_3_12, feature = "macros"))]
#[allow(clippy::bool_comparison)] // `== false` is harder to miss than !
mod opaque_tests {
    use memoffset::offset_of;
    use static_assertions::const_assert;
    use std::mem::size_of;
    use std::ops::Range;

    #[cfg(not(Py_LIMITED_API))]
    use super::test_utils::get_pyobject_size;
    use super::*;

    use crate::{prelude::*, PyClass};

    /// Check that all the type properties are as expected for the given class `T`.
    /// Unlike the static layout, the properties of a type in the opaque layout are
    /// derived entirely from `T`, not the whole [ffi::PyObject].
    fn check_opaque_type_properties<T: PyClass>(has_dict: bool, has_weakref: bool) {
        let contents_size = size_of::<PyClassObjectContents<T>>() as ffi::Py_ssize_t;
        // negative indicates 'in addition to the base class'
        assert!(PyObjectLayout::basicsize::<T>() == -contents_size);

        assert_eq!(
            PyObjectLayout::contents_offset::<T>(),
            PyObjectOffset::Relative(0),
        );

        let dict_size = size_of::<<T as PyClassImpl>::Dict>();
        if has_dict {
            assert!(dict_size > 0);
        } else {
            assert_eq!(dict_size, 0);
        }
        let expected_dict_offset_in_contents =
            offset_of!(PyClassObjectContents<T>, dict) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::dict_offset::<T>(),
            PyObjectOffset::Relative(expected_dict_offset_in_contents),
        );

        let weakref_size = size_of::<<T as PyClassImpl>::WeakRef>();
        if has_weakref {
            assert!(weakref_size > 0);
        } else {
            assert_eq!(weakref_size, 0);
        }
        let expected_weakref_offset_in_contents =
            offset_of!(PyClassObjectContents<T>, weakref) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::weaklist_offset::<T>(),
            PyObjectOffset::Relative(expected_weakref_offset_in_contents),
        );

        if has_dict {
            assert!(expected_dict_offset_in_contents < expected_weakref_offset_in_contents);
        } else {
            assert_eq!(
                expected_dict_offset_in_contents,
                expected_weakref_offset_in_contents
            );
        }
    }

    /// Test the functions calculate properties about the opaque layout without requiring an instance.
    /// The class in this test extends the default base class `PyAny` so there is 'no inheritance'.
    #[test]
    fn test_type_properties_no_inheritance() {
        #[pyclass(crate = "crate", opaque, extends=PyAny)]
        struct MyClass(#[allow(unused)] u64);
        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == true);

        check_opaque_type_properties::<MyClass>(false, false);
    }

    /// Test the functions calculate properties about the opaque layout without requiring an instance.
    /// The class in this test requires extra space for the `dict` and `weaklist` fields
    #[test]
    fn test_layout_properties_no_inheritance_optional_fields() {
        #[pyclass(crate = "crate", dict, weakref, opaque, extends=PyAny)]
        struct MyClass(#[allow(unused)] u64);
        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == true);

        check_opaque_type_properties::<MyClass>(true, true);
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_type_properties_with_inheritance_opaque_base() {
        use std::any::TypeId;

        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, opaque, extends=PyDict)]
        struct ParentClass {
            #[allow(unused)]
            parent_field: u64,
        }

        #[pyclass(crate = "crate", extends=ParentClass)]
        struct ChildClass {
            #[allow(unused)]
            child_field: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == true);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);
        assert_eq!(
            TypeId::of::<<ChildClass as PyClassImpl>::BaseType>(),
            TypeId::of::<ParentClass>()
        );
        assert_eq!(
            TypeId::of::<<ParentClass as PyClassImpl>::BaseType>(),
            TypeId::of::<PyDict>()
        );

        check_opaque_type_properties::<ParentClass>(false, false);
        check_opaque_type_properties::<ChildClass>(false, false);
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_type_properties_with_inheritance_static_base() {
        use std::any::TypeId;

        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, extends=PyDict)]
        struct ParentClass {
            #[allow(unused)]
            parent_field: u64,
        }

        #[pyclass(crate = "crate", opaque, extends=ParentClass)]
        struct ChildClass {
            #[allow(unused)]
            child_field: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);
        assert_eq!(
            TypeId::of::<<ChildClass as PyClassImpl>::BaseType>(),
            TypeId::of::<ParentClass>()
        );
        assert_eq!(
            TypeId::of::<<ParentClass as PyClassImpl>::BaseType>(),
            TypeId::of::<PyDict>()
        );

        check_opaque_type_properties::<ChildClass>(false, false);

        #[repr(C)]
        struct ParentExpectedLayout {
            native_base: ffi::PyDictObject,
            parent_contents: PyClassObjectContents<ParentClass>,
        }

        let expected_size = size_of::<ParentExpectedLayout>() as ffi::Py_ssize_t;
        assert_eq!(PyObjectLayout::basicsize::<ParentClass>(), expected_size);

        let expected_parent_contents_offset =
            offset_of!(ParentExpectedLayout, parent_contents) as ffi::Py_ssize_t;
        assert_eq!(
            PyObjectLayout::contents_offset::<ParentClass>(),
            PyObjectOffset::Absolute(expected_parent_contents_offset),
        );
    }

    /// Test the functions that operate on pyclass instances
    /// The class in this test extends the default base class `PyAny` so there is 'no inheritance'.
    #[test]
    fn test_contents_access_no_inheritance() {
        #[pyclass(crate = "crate", opaque, extends=PyAny)]
        struct MyClass {
            my_value: u64,
        }

        const_assert!(<MyClass as PyTypeInfo>::OPAQUE == true);

        Python::with_gil(|py| {
            let obj = Py::new(py, MyClass { my_value: 123 }).unwrap();
            let obj_ptr = obj.as_ptr();

            // test obtaining contents pointer normally (with GIL held)
            let contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<MyClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };

            // work around the fact that pointers are not `Send`
            let obj_ptr_int = obj_ptr as usize;
            let contents_ptr_int = contents_ptr as usize;

            // test that the contents pointer can be obtained without the GIL held
            py.allow_threads(move || {
                let obj_ptr = obj_ptr_int as *mut ffi::PyObject;
                let contents_ptr = contents_ptr_int as *mut PyClassObjectContents<MyClass>;

                // Safety: type object was created when `obj` was constructed
                let contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<MyClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                assert_eq!(contents_ptr, contents_ptr_without_gil);
            });

            // test that contents pointer matches expecations
            // the `MyClass` data has to be between the base type and the end of the PyObject.
            #[cfg(not(Py_LIMITED_API))]
            {
                let pyobject_size = get_pyobject_size::<MyClass>(py);
                let contents_range = bytes_range(
                    contents_ptr_int - obj_ptr_int,
                    size_of::<PyClassObjectContents<MyClass>>(),
                );
                assert!(contents_range.start >= size_of::<ffi::PyObject>());
                assert!(contents_range.end <= pyobject_size);
            }

            // test getting contents by reference
            let contents = unsafe {
                PyObjectLayout::get_contents::<MyClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(contents), contents_ptr);

            // test getting data pointer
            let data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<MyClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let expected_data_ptr = unsafe { (*contents_ptr).value.get() };
            assert_eq!(data_ptr, expected_data_ptr);
            assert_eq!(unsafe { (*data_ptr).my_value }, 123);

            // test getting data by reference
            let data = unsafe {
                PyObjectLayout::get_data::<MyClass>(obj.as_raw_ref(), TypeObjectStrategy::lazy(py))
            };
            assert_eq!(ptr_from_ref(data), expected_data_ptr);
        });
    }

    /// Test the functions that operate on pyclass instances.
    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_contents_access_with_inheritance_opaque_base() {
        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, opaque, extends=PyDict)]
        struct ParentClass {
            parent_value: u64,
        }

        #[pyclass(crate = "crate", extends=ParentClass)]
        struct ChildClass {
            child_value: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == true);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);

        Python::with_gil(|py| {
            let obj = Py::new(
                py,
                PyClassInitializer::from(ParentClass { parent_value: 123 }).add_subclass(
                    ChildClass {
                        child_value: "foo".to_owned(),
                    },
                ),
            )
            .unwrap();
            let obj_ptr = obj.as_ptr();

            // test obtaining contents pointer normally (with GIL held)
            let parent_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ParentClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ChildClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };

            // work around the fact that pointers are not `Send`
            let obj_ptr_int = obj_ptr as usize;
            let parent_contents_ptr_int = parent_contents_ptr as usize;
            let child_contents_ptr_int = child_contents_ptr as usize;

            // test that the contents pointer can be obtained without the GIL held
            py.allow_threads(move || {
                let obj_ptr = obj_ptr_int as *mut ffi::PyObject;
                let parent_contents_ptr =
                    parent_contents_ptr_int as *mut PyClassObjectContents<ParentClass>;
                let child_contents_ptr =
                    child_contents_ptr_int as *mut PyClassObjectContents<ChildClass>;

                // Safety: type object was created when `obj` was constructed
                let parent_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ParentClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                let child_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ChildClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                assert_eq!(parent_contents_ptr, parent_contents_ptr_without_gil);
                assert_eq!(child_contents_ptr, child_contents_ptr_without_gil);
            });

            // test that contents pointer matches expecations
            let parent_pyobject_size = get_pyobject_size::<ParentClass>(py);
            let child_pyobject_size = get_pyobject_size::<ChildClass>(py);
            assert!(child_pyobject_size > parent_pyobject_size);
            let parent_range = bytes_range(
                parent_contents_ptr_int - obj_ptr_int,
                size_of::<PyClassObjectContents<ParentClass>>(),
            );
            let child_range = bytes_range(
                child_contents_ptr_int - obj_ptr_int,
                size_of::<PyClassObjectContents<ChildClass>>(),
            );
            assert!(parent_range.start >= size_of::<ffi::PyDictObject>());
            assert!(parent_range.end <= parent_pyobject_size);
            assert!(child_range.start >= parent_range.end);
            assert!(child_range.end <= child_pyobject_size);

            // test getting contents by reference
            let parent_contents = unsafe {
                PyObjectLayout::get_contents::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents = unsafe {
                PyObjectLayout::get_contents::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_contents), parent_contents_ptr);
            assert_eq!(ptr_from_ref(child_contents), child_contents_ptr);

            // test getting data pointer
            let parent_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ParentClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let child_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ChildClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let expected_parent_data_ptr = unsafe { (*parent_contents_ptr).value.get() };
            let expected_child_data_ptr = unsafe { (*child_contents_ptr).value.get() };
            assert_eq!(parent_data_ptr, expected_parent_data_ptr);
            assert_eq!(unsafe { (*parent_data_ptr).parent_value }, 123);
            assert_eq!(child_data_ptr, expected_child_data_ptr);
            assert_eq!(unsafe { &(*child_data_ptr).child_value }, "foo");

            // test getting data by reference
            let parent_data = unsafe {
                PyObjectLayout::get_data::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_data = unsafe {
                PyObjectLayout::get_data::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_data), expected_parent_data_ptr);
            assert_eq!(ptr_from_ref(child_data), expected_child_data_ptr);
        });
    }

    /// Test the functions that operate on pyclass instances.
    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_contents_access_with_inheritance_static_base() {
        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, extends=PyDict)]
        struct ParentClass {
            parent_value: u64,
        }

        #[pyclass(crate = "crate", opaque, extends=ParentClass)]
        struct ChildClass {
            child_value: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == false);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);

        Python::with_gil(|py| {
            let obj = Py::new(
                py,
                PyClassInitializer::from(ParentClass { parent_value: 123 }).add_subclass(
                    ChildClass {
                        child_value: "foo".to_owned(),
                    },
                ),
            )
            .unwrap();
            let obj_ptr = obj.as_ptr();

            // test obtaining contents pointer normally (with GIL held)
            let parent_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ParentClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents_ptr = unsafe {
                PyObjectLayout::get_contents_ptr::<ChildClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };

            // work around the fact that pointers are not `Send`
            let obj_ptr_int = obj_ptr as usize;
            let parent_contents_ptr_int = parent_contents_ptr as usize;
            let child_contents_ptr_int = child_contents_ptr as usize;

            // test that the contents pointer can be obtained without the GIL held
            py.allow_threads(move || {
                let obj_ptr = obj_ptr_int as *mut ffi::PyObject;
                let parent_contents_ptr =
                    parent_contents_ptr_int as *mut PyClassObjectContents<ParentClass>;
                let child_contents_ptr =
                    child_contents_ptr_int as *mut PyClassObjectContents<ChildClass>;

                // Safety: type object was created when `obj` was constructed
                let parent_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ParentClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                let child_contents_ptr_without_gil = unsafe {
                    PyObjectLayout::get_contents_ptr::<ChildClass>(
                        obj_ptr,
                        TypeObjectStrategy::assume_init(),
                    )
                };
                assert_eq!(parent_contents_ptr, parent_contents_ptr_without_gil);
                assert_eq!(child_contents_ptr, child_contents_ptr_without_gil);
            });

            // test that contents pointer matches expecations
            let parent_pyobject_size = get_pyobject_size::<ParentClass>(py);
            let child_pyobject_size = get_pyobject_size::<ChildClass>(py);
            assert!(child_pyobject_size > parent_pyobject_size);
            let parent_range = bytes_range(
                parent_contents_ptr_int - obj_ptr_int,
                size_of::<PyClassObjectContents<ParentClass>>(),
            );
            let child_range = bytes_range(
                child_contents_ptr_int - obj_ptr_int,
                size_of::<PyClassObjectContents<ChildClass>>(),
            );
            assert!(parent_range.start >= size_of::<ffi::PyDictObject>());
            assert!(parent_range.end <= parent_pyobject_size);
            assert!(child_range.start >= parent_range.end);
            assert!(child_range.end <= child_pyobject_size);

            // test getting contents by reference
            let parent_contents = unsafe {
                PyObjectLayout::get_contents::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_contents = unsafe {
                PyObjectLayout::get_contents::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_contents), parent_contents_ptr);
            assert_eq!(ptr_from_ref(child_contents), child_contents_ptr);

            // test getting data pointer
            let parent_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ParentClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let child_data_ptr = unsafe {
                PyObjectLayout::get_data_ptr::<ChildClass>(obj_ptr, TypeObjectStrategy::lazy(py))
            };
            let expected_parent_data_ptr = unsafe { (*parent_contents_ptr).value.get() };
            let expected_child_data_ptr = unsafe { (*child_contents_ptr).value.get() };
            assert_eq!(parent_data_ptr, expected_parent_data_ptr);
            assert_eq!(unsafe { (*parent_data_ptr).parent_value }, 123);
            assert_eq!(child_data_ptr, expected_child_data_ptr);
            assert_eq!(unsafe { &(*child_data_ptr).child_value }, "foo");

            // test getting data by reference
            let parent_data = unsafe {
                PyObjectLayout::get_data::<ParentClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            let child_data = unsafe {
                PyObjectLayout::get_data::<ChildClass>(
                    obj.as_raw_ref(),
                    TypeObjectStrategy::lazy(py),
                )
            };
            assert_eq!(ptr_from_ref(parent_data), expected_parent_data_ptr);
            assert_eq!(ptr_from_ref(child_data), expected_child_data_ptr);
        });
    }

    /// Test that `Drop::drop` is called for pyclasses
    #[test]
    fn test_destructor_called() {
        use std::sync::{Arc, Mutex};

        let deallocations: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        #[pyclass(crate = "crate", subclass, opaque)]
        struct ParentClass(Arc<Mutex<Vec<String>>>);

        impl Drop for ParentClass {
            fn drop(&mut self) {
                self.0.lock().unwrap().push("ParentClass".to_owned());
            }
        }

        #[pyclass(crate = "crate", extends = ParentClass)]
        struct ChildClass(Arc<Mutex<Vec<String>>>);

        impl Drop for ChildClass {
            fn drop(&mut self) {
                self.0.lock().unwrap().push("ChildClass".to_owned());
            }
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == true);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);

        Python::with_gil(|py| {
            let obj = Py::new(
                py,
                PyClassInitializer::from(ParentClass(deallocations.clone()))
                    .add_subclass(ChildClass(deallocations.clone())),
            )
            .unwrap();
            assert!(deallocations.lock().unwrap().is_empty());
            drop(obj);
        });

        assert_eq!(
            deallocations.lock().unwrap().as_slice(),
            &["ChildClass", "ParentClass"]
        );
    }

    #[test]
    #[should_panic(expected = "OpaqueClassNeverConstructed not initialized")]
    fn test_panic_when_incorrectly_assume_initialized() {
        #[pyclass(crate = "crate", opaque)]
        struct OpaqueClassNeverConstructed;

        const_assert!(<OpaqueClassNeverConstructed as PyTypeInfo>::OPAQUE);

        let obj = Python::with_gil(|py| py.None());

        assert!(OpaqueClassNeverConstructed::try_get_type_object_raw().is_none());
        unsafe {
            PyObjectLayout::get_contents_ptr::<OpaqueClassNeverConstructed>(
                obj.as_ptr(),
                TypeObjectStrategy::assume_init(),
            );
        }
    }

    #[test]
    #[cfg(all(debug_assertions, not(Py_LIMITED_API)))]
    #[should_panic(expected = "the object is not an instance of")]
    fn test_panic_when_incorrect_type() {
        use crate::types::PyDict;

        #[pyclass(crate = "crate", subclass, opaque, extends=PyDict)]
        struct ParentClass {
            #[allow(unused)]
            parent_value: u64,
        }

        #[pyclass(crate = "crate", extends=ParentClass)]
        struct ChildClass {
            #[allow(unused)]
            child_value: String,
        }

        const_assert!(<ParentClass as PyTypeInfo>::OPAQUE == true);
        const_assert!(<ChildClass as PyTypeInfo>::OPAQUE == true);

        Python::with_gil(|py| {
            let obj = Py::new(py, ParentClass { parent_value: 123 }).unwrap();
            let obj_ptr = obj.as_ptr();

            unsafe {
                PyObjectLayout::get_contents_ptr::<ChildClass>(
                    obj_ptr,
                    TypeObjectStrategy::lazy(py),
                )
            };
        });
    }

    /// Create a range from a start and size instead of a start and end
    #[allow(unused)]
    fn bytes_range(start: usize, size: usize) -> Range<usize> {
        Range {
            start,
            end: start + size,
        }
    }
}

#[cfg(all(test, not(Py_3_12), feature = "macros"))]
mod opaque_fail_tests {
    use crate::{
        prelude::*,
        types::{PyDict, PyTuple, PyType},
        PyTypeInfo,
    };

    #[pyclass(crate = "crate", extends=PyType)]
    #[derive(Default)]
    struct Metaclass;

    #[pymethods(crate = "crate")]
    impl Metaclass {
        #[pyo3(signature = (*_args, **_kwargs))]
        fn __init__(&mut self, _args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) {}
    }

    /// PyType uses the opaque layout. While explicitly using `#[pyclass(opaque)]` can be caught at compile time,
    /// it is also possible to create a pyclass that uses the opaque layout by extending an opaque native type.
    #[test]
    #[should_panic(
        expected = "The opaque object layout (used by pyo3::pycell::layout::opaque_fail_tests::Metaclass) is not supported until python 3.12"
    )]
    fn test_panic_at_construction_inherit_opaque() {
        Python::with_gil(|py| {
            Py::new(py, Metaclass).unwrap();
        });
    }

    #[test]
    #[should_panic(
        expected = "The opaque object layout (used by pyo3::pycell::layout::opaque_fail_tests::Metaclass) is not supported until python 3.12"
    )]
    fn test_panic_at_type_construction_inherit_opaque() {
        Python::with_gil(|py| {
            <Metaclass as PyTypeInfo>::type_object(py);
        });
    }
}

#[cfg(test)]
mod test_utils {
    #[cfg(not(Py_LIMITED_API))]
    use crate::{ffi, PyClass, PyTypeInfo, Python};

    /// The size in bytes of a [ffi::PyObject] of the type `T`
    #[cfg(not(Py_LIMITED_API))]
    #[allow(unused)]
    pub fn get_pyobject_size<T: PyClass>(py: Python<'_>) -> usize {
        let typ = <T as PyTypeInfo>::type_object(py);
        let raw_typ = typ.as_ptr().cast::<ffi::PyTypeObject>();
        let size = unsafe { (*raw_typ).tp_basicsize };
        usize::try_from(size).expect("size should be a valid usize")
    }
}
