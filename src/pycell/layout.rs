#![allow(missing_docs)]
//! Crate-private implementation of how PyClassObjects are laid out in memory and how to access data from raw PyObjects

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::addr_of_mut;

use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef, PyObjectOffset,
};
use crate::internal::get_slot::TP_FREE;
use crate::pycell::borrow_checker::{GetBorrowChecker, PyClassBorrowChecker};
use crate::type_object::PyNativeType;
use crate::types::PyType;
use crate::{ffi, PyTypeInfo, Python};

#[cfg(not(Py_LIMITED_API))]
use crate::types::PyTypeMethods;

use super::borrow_checker::PyClassMutability;
use super::{ptr_from_ref, PyBorrowError};

/// The data of a `ffi::PyObject` specifically relating to type `T`.
///
/// In an inheritance hierarchy where `#[pyclass(extends=PyDict)] struct A;` and `#[pyclass(extends=A)] struct B;`
/// a `ffi::PyObject` of type `B` has separate memory for `ffi::PyDictObject` (the base native type) and
/// `PyClassObjectContents<A>` and `PyClassObjectContents<B>`. The memory associated with `A` or `B` can be obtained
/// using `PyObjectLayout::get_contents::<T>()` (where `T=A` or `T=B`).
#[repr(C)]
pub(crate) struct PyClassObjectContents<T: PyClassImpl> {
    /// The data associated with the user-defined struct annotated with `#[pyclass]`
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    /// A pointer to a `PyObject` if `T` is annotated with `#[pyclass(dict)]` and a zero-sized field otherwise.
    pub(crate) dict: T::Dict,
    /// A pointer to a `PyObject` if `T` is annotated with `#[pyclass(weakref)]` and a zero-sized field otherwise.
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

/// Functions for working with `PyObjects` recursively by re-interpreting the object
/// as being an instance of the most derived class through each base class until
/// the `BaseNativeType` is reached.
///
/// E.g. if `#[pyclass(extends=PyDict)] struct A;` and `#[pyclass(extends=A)] struct B;`
/// then calling a method on a PyObject of type `B` will call the method for `B`, then `A`, then `PyDict`.
#[doc(hidden)]
pub trait PyObjectRecursiveOperations {
    /// `PyTypeInfo::type_object_raw()` may create type objects lazily.
    /// This method ensures that the type objects for all ancestor types of the provided object.
    unsafe fn ensure_type_objects_initialized(py: Python<'_>);

    /// Call `PyClassThreadChecker::ensure` on all ancestor types of the provided object.
    ///
    /// # Safety
    ///
    /// - if the object uses the opaque layout, all ancestor types must be initialized beforehand.
    unsafe fn ensure_threadsafe(obj: &ffi::PyObject);

    /// Call `PyClassThreadChecker::check` on all ancestor types of the provided object.
    ///
    /// # Safety
    ///
    /// - if the object uses the opaque layout, all ancestor types must be initialized beforehand.
    unsafe fn check_threadsafe(obj: &ffi::PyObject) -> Result<(), PyBorrowError>;

    /// Cleanup then free the memory for `obj`.
    ///
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject);
}

/// Used to fill out `PyClassBaseType::RecursiveOperations` for instances of `PyClass`
pub struct PyClassRecursiveOperations<T>(PhantomData<T>);

impl<T: PyClassImpl + PyTypeInfo> PyObjectRecursiveOperations for PyClassRecursiveOperations<T> {
    unsafe fn ensure_type_objects_initialized(py: Python<'_>) {
        let _ = <T as PyTypeInfo>::type_object_raw(py);
        <T::BaseType as PyClassBaseType>::RecursiveOperations::ensure_type_objects_initialized(py);
    }

    unsafe fn ensure_threadsafe(obj: &ffi::PyObject) {
        let type_provider = AssumeInitializedTypeProvider::new();
        let contents = PyObjectLayout::get_contents::<T, _>(obj, type_provider);
        contents.thread_checker.ensure();
        <T::BaseType as PyClassBaseType>::RecursiveOperations::ensure_threadsafe(obj);
    }

    unsafe fn check_threadsafe(obj: &ffi::PyObject) -> Result<(), PyBorrowError> {
        let type_provider = AssumeInitializedTypeProvider::new();
        let contents = PyObjectLayout::get_contents::<T, _>(obj, type_provider);
        if !contents.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        <T::BaseType as PyClassBaseType>::RecursiveOperations::check_threadsafe(obj)
    }

    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let contents =
            &mut *PyObjectLayout::get_contents_ptr::<T, _>(obj, LazyTypeProvider::new(py));
        contents.dealloc(py, obj);
        <T::BaseType as PyClassBaseType>::RecursiveOperations::deallocate(py, obj);
    }
}

/// Used to fill out `PyClassBaseType::RecursiveOperations` for native types
pub struct PyNativeTypeRecursiveOperations<T>(PhantomData<T>);

impl<T: PyNativeType + PyTypeInfo> PyObjectRecursiveOperations
    for PyNativeTypeRecursiveOperations<T>
{
    unsafe fn ensure_type_objects_initialized(py: Python<'_>) {
        let _ = <T as PyTypeInfo>::type_object_raw(py);
    }

    unsafe fn ensure_threadsafe(_obj: &ffi::PyObject) {}

    unsafe fn check_threadsafe(_obj: &ffi::PyObject) -> Result<(), PyBorrowError> {
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
    /// - obj must be a valid pointer to an instance of the type at `type_ptr` or a subclass.
    /// - obj must not be used after this call (as it will be freed).
    unsafe fn deallocate(py: Python<'_>, obj: *mut ffi::PyObject) {
        // the `BaseNativeType` of the object
        let type_ptr = <T as PyTypeInfo>::type_object_raw(py);

        // FIXME: there is potentially subtle issues here if the base is overwritten at runtime? To be investigated.

        // the 'most derived class' of `obj`. i.e. the result of calling `type(obj)`.
        let actual_type = PyType::from_borrowed_type_ptr(py, ffi::Py_TYPE(obj));

        // TODO(matt): is this correct?
        // For `#[pyclass]` types which inherit from PyAny or PyType, we can just call tp_free
        let is_base_object = type_ptr == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type);
        let is_metaclass = type_ptr == std::ptr::addr_of_mut!(ffi::PyType_Type);
        if is_base_object || is_metaclass {
            let tp_free = actual_type
                .get_slot(TP_FREE)
                .expect("base type should have tp_free");
            return tp_free(obj.cast());
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

/// Utilities for working with `PyObject` objects that utilise [PEP 697](https://peps.python.org/pep-0697/).
#[doc(hidden)]
pub(crate) mod opaque_layout {
    use super::PyClassObjectContents;
    use super::TypeObjectProvider;
    use crate::{ffi, impl_::pyclass::PyClassImpl, PyTypeInfo};

    #[cfg(Py_3_12)]
    pub fn get_contents_ptr<T: PyClassImpl + PyTypeInfo, P: TypeObjectProvider<T>>(
        obj: *mut ffi::PyObject,
        type_provider: P,
    ) -> *mut PyClassObjectContents<T> {
        #[cfg(Py_3_12)]
        {
            let type_obj = type_provider.get_type_object();
            assert!(!type_obj.is_null(), "type object is NULL");
            let pointer = unsafe { ffi::PyObject_GetTypeData(obj, type_obj) };
            assert!(!pointer.is_null(), "pointer to pyclass data returned NULL");
            pointer.cast()
        }

        #[cfg(not(Py_3_12))]
        panic_unsupported();
    }

    #[inline(always)]
    #[cfg(not(Py_3_12))]
    fn panic_unsupported() {
        panic!("opaque layout not supported until python 3.12");
    }
}

/// Utilities for working with `PyObject` objects that utilise the standard layout for python extensions,
/// where the base class is placed at the beginning of a `repr(C)` struct.
#[doc(hidden)]
pub(crate) mod static_layout {
    use crate::{
        impl_::pyclass::{PyClassBaseType, PyClassImpl},
        type_object::{PyLayout, PySizedLayout},
    };

    use super::PyClassObjectContents;

    // The layout of a `PyObject` that uses the static layout
    #[repr(C)]
    pub struct PyStaticClassLayout<T: PyClassImpl> {
        pub(crate) ob_base: <T::BaseType as PyClassBaseType>::StaticLayout,
        pub(crate) contents: PyClassObjectContents<T>,
    }

    unsafe impl<T: PyClassImpl> PyLayout<T> for PyStaticClassLayout<T> {}

    /// Base layout of PyClassObject with a known sized base type.
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
    /// since nothing can actually be read by dereferencing.
    unsafe impl<T> PyLayout<T> for InvalidStaticLayout {}
}

/// A trait for obtaining a `*mut ffi::PyTypeObject` pointer describing `T` for use with `PyObjectLayout` functions.
///
/// `PyTypeInfo::type_object_raw()` requires the GIL to be held because it may lazily construct the type object.
/// Some situations require that the GIL is not held so `PyObjectLayout` cannot call this method directly.
/// The different solutions to this have different trade-offs so the caller can decide using a `TypeObjectProvider`.
pub trait TypeObjectProvider<T: PyTypeInfo> {
    fn get_type_object(&self) -> *mut ffi::PyTypeObject;
}

/// Hold the GIL and only obtain/construct the type object if required.
///
/// Since the type object is only required for accessing opaque objects, this option has the best
/// performance but it requires the GIL being held.
pub struct LazyTypeProvider<'py, T: PyTypeInfo>(PhantomData<&'py T>);
impl<'py, T: PyTypeInfo> LazyTypeProvider<'py, T> {
    pub fn new(_py: Python<'py>) -> Self {
        Self(PhantomData)
    }
}
impl<'py, T: PyTypeInfo> TypeObjectProvider<T> for LazyTypeProvider<'py, T> {
    fn get_type_object(&self) -> *mut ffi::PyTypeObject {
        let py: Python<'py> = unsafe { Python::assume_gil_acquired() };
        T::type_object_raw(py)
    }
}

/// Will assume that `PyTypeInfo::type_object_raw()` has been called so the type object
/// is cached and can be obtained without holding the GIL.
pub struct AssumeInitializedTypeProvider<T: PyTypeInfo>(PhantomData<T>);
impl<T: PyTypeInfo> AssumeInitializedTypeProvider<T> {
    pub unsafe fn new() -> Self {
        Self(PhantomData)
    }
}
impl<T: PyTypeInfo> TypeObjectProvider<T> for AssumeInitializedTypeProvider<T> {
    fn get_type_object(&self) -> *mut ffi::PyTypeObject {
        T::try_get_type_object_raw().unwrap_or_else(|| {
            panic!(
                "type object for {} not initialized",
                std::any::type_name::<T>()
            )
        })
    }
}

/// Functions for working with `PyObject`s
pub(crate) struct PyObjectLayout {}

impl PyObjectLayout {
    /// Obtain a pointer to the contents of a `PyObject` of type `T`.
    ///
    /// Safety: the provided object must be valid and have the layout indicated by `T`
    pub(crate) unsafe fn get_contents_ptr<T: PyClassImpl + PyTypeInfo, P: TypeObjectProvider<T>>(
        obj: *mut ffi::PyObject,
        type_provider: P,
    ) -> *mut PyClassObjectContents<T> {
        debug_assert!(!obj.is_null());
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            opaque_layout::get_contents_ptr(obj, type_provider)
        } else {
            let obj: *mut static_layout::PyStaticClassLayout<T> = obj.cast();
            // indicates `ob_base` has type InvalidBaseLayout
            debug_assert_ne!(
                std::mem::offset_of!(static_layout::PyStaticClassLayout<T>, contents),
                0,
                "invalid ob_base found"
            );
            addr_of_mut!((*obj).contents)
        }
    }

    pub(crate) unsafe fn get_contents<T: PyClassImpl + PyTypeInfo, P: TypeObjectProvider<T>>(
        obj: &ffi::PyObject,
        type_provider: P,
    ) -> &PyClassObjectContents<T> {
        &*PyObjectLayout::get_contents_ptr::<T, P>(ptr_from_ref(obj).cast_mut(), type_provider)
            .cast_const()
    }

    /// obtain a pointer to the pyclass struct of a `PyObject` of type `T`.
    ///
    /// Safety: the provided object must be valid and have the layout indicated by `T`
    pub(crate) unsafe fn get_data_ptr<T: PyClassImpl + PyTypeInfo, P: TypeObjectProvider<T>>(
        obj: *mut ffi::PyObject,
        type_provider: P,
    ) -> *mut T {
        let contents = PyObjectLayout::get_contents_ptr::<T, P>(obj, type_provider);
        (*contents).value.get()
    }

    pub(crate) unsafe fn get_data<T: PyClassImpl + PyTypeInfo, P: TypeObjectProvider<T>>(
        obj: &ffi::PyObject,
        type_provider: P,
    ) -> &T {
        &*PyObjectLayout::get_data_ptr::<T, P>(ptr_from_ref(obj).cast_mut(), type_provider)
    }

    pub(crate) unsafe fn get_borrow_checker<'o, T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: &'o ffi::PyObject,
    ) -> &'o <T::PyClassMutability as PyClassMutability>::Checker {
        if T::OPAQUE {
            PyClassRecursiveOperations::<T>::ensure_type_objects_initialized(py);
        }
        T::PyClassMutability::borrow_checker(obj)
    }

    pub(crate) unsafe fn ensure_threadsafe<T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: &ffi::PyObject,
    ) {
        if T::OPAQUE {
            PyClassRecursiveOperations::<T>::ensure_type_objects_initialized(py);
        }
        PyClassRecursiveOperations::<T>::ensure_threadsafe(obj);
    }

    /// Clean up then free the memory associated with `obj`.
    ///
    /// See [tp_dealloc docs](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dealloc)
    pub(crate) fn deallocate<T: PyClassImpl + PyTypeInfo>(py: Python<'_>, obj: *mut ffi::PyObject) {
        unsafe {
            PyClassRecursiveOperations::<T>::deallocate(py, obj);
        };
    }

    /// Clean up then free the memory associated with `obj`.
    ///
    /// Use instead of `deallocate()` if `T` has the `Py_TPFLAGS_HAVE_GC` flag set.
    ///
    /// See [tp_dealloc docs](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dealloc)
    pub(crate) fn deallocate_with_gc<T: PyClassImpl + PyTypeInfo>(
        py: Python<'_>,
        obj: *mut ffi::PyObject,
    ) {
        unsafe {
            // TODO(matt): verify T has flag set
            #[cfg(not(PyPy))]
            {
                ffi::PyObject_GC_UnTrack(obj.cast());
            }
            PyClassRecursiveOperations::<T>::deallocate(py, obj);
        };
    }

    /// Used to set `PyType_Spec::basicsize` when creating a `PyTypeObject` for `T`
    /// ([docs](https://docs.python.org/3/c-api/type.html#c.PyType_Spec.basicsize))
    pub(crate) fn basicsize<T: PyClassImpl>() -> ffi::Py_ssize_t {
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            #[cfg(Py_3_12)]
            {
                // negative to indicate 'extra' space that python will allocate
                // specifically for `T` excluding the base class
                -usize_to_py_ssize(std::mem::size_of::<PyClassObjectContents<T>>())
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported();
        } else {
            usize_to_py_ssize(std::mem::size_of::<static_layout::PyStaticClassLayout<T>>())
        }
    }

    /// Gets the offset of the contents from the start of the struct in bytes.
    pub(crate) fn contents_offset<T: PyClassImpl>() -> PyObjectOffset {
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(0)
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported();
        } else {
            PyObjectOffset::Absolute(usize_to_py_ssize(memoffset::offset_of!(
                static_layout::PyStaticClassLayout<T>,
                contents
            )))
        }
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    pub(crate) fn dict_offset<T: PyClassImpl>() -> PyObjectOffset {
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
                    PyClassObjectContents<T>,
                    dict
                )))
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported();
        } else {
            let offset = memoffset::offset_of!(static_layout::PyStaticClassLayout<T>, contents)
                + memoffset::offset_of!(PyClassObjectContents<T>, dict);

            PyObjectOffset::Absolute(usize_to_py_ssize(offset))
        }
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    pub(crate) fn weaklist_offset<T: PyClassImpl>() -> PyObjectOffset {
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            #[cfg(Py_3_12)]
            {
                PyObjectOffset::Relative(usize_to_py_ssize(memoffset::offset_of!(
                    PyClassObjectContents<T>,
                    weakref
                )))
            }

            #[cfg(not(Py_3_12))]
            opaque_layout::panic_unsupported();
        } else {
            let offset = memoffset::offset_of!(static_layout::PyStaticClassLayout<T>, contents)
                + memoffset::offset_of!(PyClassObjectContents<T>, weakref);

            PyObjectOffset::Absolute(usize_to_py_ssize(offset))
        }
    }
}

/// Py_ssize_t may not be equal to isize on all platforms
fn usize_to_py_ssize(value: usize) -> ffi::Py_ssize_t {
    #[allow(clippy::useless_conversion)]
    value.try_into().expect("value should fit in Py_ssize_t")
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;

    use crate::prelude::*;

    #[pyclass(crate = "crate", subclass)]
    struct BaseWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithData(#[allow(unused)] u64);

    #[pyclass(crate = "crate", extends = BaseWithData)]
    struct ChildWithoutData;

    #[test]
    fn test_inherited_size() {
        let base_size = PyObjectLayout::basicsize::<BaseWithData>();
        assert!(base_size > 0); // negative indicates variable sized
        assert_eq!(base_size, PyObjectLayout::basicsize::<ChildWithoutData>());
        assert!(base_size < PyObjectLayout::basicsize::<ChildWithData>());
    }

    #[test]
    fn test_invalid_base() {
        assert_eq!(std::mem::size_of::<static_layout::InvalidStaticLayout>(), 0);

        #[repr(C)]
        struct InvalidLayout {
            ob_base: static_layout::InvalidStaticLayout,
            contents: u8,
        }

        assert_eq!(std::mem::offset_of!(InvalidLayout, contents), 0);
    }
}
