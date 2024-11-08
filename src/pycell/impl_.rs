#![allow(missing_docs)]
//! Crate-private implementation of PyClassObject

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef, PyObjectOffset,
};
use crate::internal::get_slot::TP_FREE;
use crate::type_object::{PyLayout, PySizedLayout};
use crate::types::PyType;
use crate::{ffi, PyClass, PyTypeInfo, Python};

#[cfg(not(Py_LIMITED_API))]
use crate::types::PyTypeMethods;

use super::{PyBorrowError, PyBorrowMutError};

pub trait PyClassMutability {
    // The storage for this inheritance layer. Only the first mutable class in
    // an inheritance hierarchy needs to store the borrow flag.
    type Storage: PyClassBorrowChecker;
    // The borrow flag needed to implement this class' mutability. Empty until
    // the first mutable class, at which point it is BorrowChecker and will be
    // for all subclasses.
    type Checker: PyClassBorrowChecker;
    type ImmutableChild: PyClassMutability;
    type MutableChild: PyClassMutability;
}

pub struct ImmutableClass(());
pub struct MutableClass(());
pub struct ExtendsMutableAncestor<M: PyClassMutability>(PhantomData<M>);

impl PyClassMutability for ImmutableClass {
    type Storage = EmptySlot;
    type Checker = EmptySlot;
    type ImmutableChild = ImmutableClass;
    type MutableChild = MutableClass;
}

impl PyClassMutability for MutableClass {
    type Storage = BorrowChecker;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

impl<M: PyClassMutability> PyClassMutability for ExtendsMutableAncestor<M> {
    type Storage = EmptySlot;
    type Checker = BorrowChecker;
    type ImmutableChild = ExtendsMutableAncestor<ImmutableClass>;
    type MutableChild = ExtendsMutableAncestor<MutableClass>;
}

#[derive(Debug)]
struct BorrowFlag(AtomicUsize);

impl BorrowFlag {
    pub(crate) const UNUSED: usize = 0;
    const HAS_MUTABLE_BORROW: usize = usize::MAX;
    fn increment(&self) -> Result<(), PyBorrowError> {
        let mut value = self.0.load(Ordering::Relaxed);
        loop {
            if value == BorrowFlag::HAS_MUTABLE_BORROW {
                return Err(PyBorrowError { _private: () });
            }
            match self.0.compare_exchange(
                // only increment if the value hasn't changed since the
                // last atomic load
                value,
                value + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(..) => {
                    // value has been successfully incremented, we need an acquire fence
                    // so that data this borrow flag protects can be read safely in this thread
                    std::sync::atomic::fence(Ordering::Acquire);
                    break Ok(());
                }
                Err(changed_value) => {
                    // value changed under us, need to try again
                    value = changed_value;
                }
            }
        }
    }
    fn decrement(&self) {
        // impossible to get into a bad state from here so relaxed
        // ordering is fine, the decrement only needs to eventually
        // be visible
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct EmptySlot(());
pub struct BorrowChecker(BorrowFlag);

pub trait PyClassBorrowChecker {
    /// Initial value for self
    fn new() -> Self;
    /// Increments immutable borrow count, if possible
    fn try_borrow(&self) -> Result<(), PyBorrowError>;
    /// Decrements immutable borrow count
    fn release_borrow(&self);
    /// Increments mutable borrow count, if possible
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError>;
    /// Decrements mutable borrow count
    fn release_borrow_mut(&self);
}

impl PyClassBorrowChecker for EmptySlot {
    #[inline]
    fn new() -> Self {
        EmptySlot(())
    }

    #[inline]
    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        Ok(())
    }

    #[inline]
    fn release_borrow(&self) {}

    #[inline]
    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        unreachable!()
    }

    #[inline]
    fn release_borrow_mut(&self) {
        unreachable!()
    }
}

impl PyClassBorrowChecker for BorrowChecker {
    #[inline]
    fn new() -> Self {
        Self(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)))
    }

    fn try_borrow(&self) -> Result<(), PyBorrowError> {
        self.0.increment()
    }

    fn release_borrow(&self) {
        self.0.decrement();
    }

    fn try_borrow_mut(&self) -> Result<(), PyBorrowMutError> {
        let flag = &self.0;
        match flag.0.compare_exchange(
            // only allowed to transition to mutable borrow if the reference is
            // currently unused
            BorrowFlag::UNUSED,
            BorrowFlag::HAS_MUTABLE_BORROW,
            // On success, reading the flag and updating its state are an atomic
            // operation
            Ordering::AcqRel,
            // It doesn't matter precisely when the failure gets turned
            // into an error
            Ordering::Relaxed,
        ) {
            Ok(..) => Ok(()),
            Err(..) => Err(PyBorrowMutError { _private: () }),
        }
    }

    fn release_borrow_mut(&self) {
        self.0 .0.store(BorrowFlag::UNUSED, Ordering::Release)
    }
}

pub trait GetBorrowChecker<T: PyClassImpl> {
    fn borrow_checker(obj: &ffi::PyObject)
        -> &<T::PyClassMutability as PyClassMutability>::Checker;
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for MutableClass {
    fn borrow_checker(obj: &ffi::PyObject) -> &BorrowChecker {
        let obj = (obj as *const ffi::PyObject).cast_mut();
        let contents = unsafe { PyObjectLayout::get_contents_ptr::<T>(obj) };
        unsafe { &(*contents).borrow_checker }
    }
}

impl<T: PyClassImpl<PyClassMutability = Self>> GetBorrowChecker<T> for ImmutableClass {
    fn borrow_checker(obj: &ffi::PyObject) -> &EmptySlot {
        let obj = (obj as *const ffi::PyObject).cast_mut();
        let contents = unsafe { PyObjectLayout::get_contents_ptr::<T>(obj) };
        unsafe { &(*contents).borrow_checker }
    }
}

impl<T, M> GetBorrowChecker<T> for ExtendsMutableAncestor<M>
where
    T: PyClassImpl<PyClassMutability = Self>,
    M: PyClassMutability,
    T::BaseType: PyClassImpl,
    <T::BaseType as PyClassImpl>::PyClassMutability: PyClassMutability<Checker = BorrowChecker>,
{
    fn borrow_checker(obj: &ffi::PyObject) -> &BorrowChecker {
        // the same PyObject pointer can be re-interpreted as the base/parent type
        <<T::BaseType as PyClassImpl>::PyClassMutability as GetBorrowChecker<T::BaseType>>::borrow_checker(obj)
    }
}

/// functionality common to all PyObjects regardless of the layout
#[doc(hidden)]
pub trait PyClassObjectBaseLayout<T>: PyLayout<T> {
    fn ensure_threadsafe(&self);
    fn check_threadsafe(&self) -> Result<(), PyBorrowError>;
    /// Implementation of tp_dealloc.
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject);
}

/// Allow [PyClassObjectLayout] to have public visibility without leaking the structure of [PyClassObjectContents].
#[doc(hidden)]
#[repr(transparent)]
pub struct WrappedPyClassObjectContents<T: PyClassImpl>(pub(crate) PyClassObjectContents<T>);

impl<'a, T: PyClassImpl> From<&'a PyClassObjectContents<T>>
    for &'a WrappedPyClassObjectContents<T>
{
    fn from(value: &'a PyClassObjectContents<T>) -> &'a WrappedPyClassObjectContents<T> {
        // Safety: Wrapped struct must use repr(transparent)
        unsafe { std::mem::transmute(value) }
    }
}

impl<'a, T: PyClassImpl> From<&'a mut PyClassObjectContents<T>>
    for &'a mut WrappedPyClassObjectContents<T>
{
    fn from(value: &'a mut PyClassObjectContents<T>) -> &'a mut WrappedPyClassObjectContents<T> {
        // Safety: Wrapped struct must use repr(transparent)
        unsafe { std::mem::transmute(value) }
    }
}

/// Functionality required for creating and managing the memory associated with a pyclass annotated struct.
#[doc(hidden)]
#[cfg_attr(
    all(diagnostic_namespace),
    diagnostic::on_unimplemented(
        message = "the class layout is not valid",
        label = "required for `#[pyclass(extends=...)]`",
        note = "the python version being built against influences which layouts are valid",
    )
)]
pub trait PyClassObjectLayout<T: PyClassImpl>: PyClassObjectBaseLayout<T> {
    /// obtain a reference to the structure that contains the pyclass struct and associated metadata.
    fn contents(&self) -> &WrappedPyClassObjectContents<T>;
}

/// Implementation of `tp_dealloc`.
/// # Safety
/// - obj must be a valid pointer to an instance of the type at `type_ptr` or a subclass.
/// - obj must not be used after this call (as it will be freed).
unsafe fn tp_dealloc(py: Python<'_>, obj: *mut ffi::PyObject, type_ptr: *mut ffi::PyTypeObject) {
    // FIXME: there is potentially subtle issues here if the base is overwritten
    // at runtime? To be investigated.
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

#[repr(C)]
pub(crate) struct PyClassObjectContents<T: PyClassImpl> {
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    pub(crate) dict: T::Dict,
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

/// The layout of a PyClassObject with a known sized base class.
#[repr(C)]
pub struct PyStaticClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    contents: PyClassObjectContents<T>,
}

impl<T: PyClassImpl> PyClassObjectLayout<T> for PyStaticClassObject<T> {
    fn contents(&self) -> &WrappedPyClassObjectContents<T> {
        (&self.contents).into()
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyStaticClassObject<T> {}
impl<T: PyClass> PySizedLayout<T> for PyStaticClassObject<T> {}

impl<T: PyClassImpl> PyClassObjectBaseLayout<T> for PyStaticClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents.thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        self.ob_base.check_threadsafe()
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let contents = unsafe { PyObjectLayout::get_contents_ptr::<T>(slf) };
        (*contents).dealloc(py, slf);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

/// A layout for a PyClassObject with an unknown sized base type.
///
/// Utilises [PEP-697](https://peps.python.org/pep-0697/)
#[doc(hidden)]
#[repr(C)]
pub struct PyVariableClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
}

impl<T: PyClassImpl> PyVariableClassObject<T> {
    #[cfg(Py_3_12)]
    fn get_contents_of_obj(obj: *mut ffi::PyObject) -> *mut WrappedPyClassObjectContents<T> {
        // https://peps.python.org/pep-0697/
        let type_obj = unsafe { ffi::Py_TYPE(obj) };
        let pointer = unsafe { ffi::PyObject_GetTypeData(obj, type_obj) };
        pointer as *mut WrappedPyClassObjectContents<T>
    }

    #[cfg(Py_3_12)]
    fn get_contents_ptr(&self) -> *mut WrappedPyClassObjectContents<T> {
        Self::get_contents_of_obj(self as *const PyVariableClassObject<T> as *mut ffi::PyObject)
    }
}

#[cfg(Py_3_12)]
impl<T: PyClassImpl> PyClassObjectLayout<T> for PyVariableClassObject<T> {
    fn contents(&self) -> &WrappedPyClassObjectContents<T> {
        unsafe { self.get_contents_ptr().cast_const().as_ref() }
            .expect("should be able to cast PyClassObjectContents pointer")
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyVariableClassObject<T> {}

#[cfg(Py_3_12)]
impl<T: PyClassImpl> PyClassObjectBaseLayout<T> for PyVariableClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents().0.thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents().0.thread_checker.check() {
            return Err(PyBorrowError { _private: () });
        }
        self.ob_base.check_threadsafe()
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let contents = unsafe { PyObjectLayout::get_contents_ptr::<T>(slf) };
        (*contents).dealloc(py, slf);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

/// Py_ssize_t may not be equal to isize on all platforms
fn usize_to_py_ssize(value: usize) -> ffi::Py_ssize_t {
    #[allow(clippy::useless_conversion)]
    value.try_into().expect("value should fit in Py_ssize_t")
}

/// Utilities for working with `PyObject` objects that utilise [PEP 697](https://peps.python.org/pep-0697/).
#[doc(hidden)]
pub(crate) mod opaque_layout {
    use super::{tp_dealloc, PyClassObjectBaseLayout, PyClassObjectContents};
    use crate::impl_::pyclass::PyClassImpl;
    use crate::pycell::PyBorrowError;
    use crate::type_object::PyLayout;
    use crate::{ffi, PyTypeInfo, Python};

    /// Base layout of `PyClassObject` with an unknown sized base type.
    /// Corresponds to [PyVarObject](https://docs.python.org/3/c-api/structures.html#c.PyVarObject) from the C API.
    #[doc(hidden)]
    #[repr(C)]
    pub struct PyVariableClassObjectBase {
        ob_base: ffi::PyVarObject,
    }

    unsafe impl<T: PyTypeInfo> PyLayout<T> for PyVariableClassObjectBase {}

    impl<T: PyTypeInfo> PyClassObjectBaseLayout<T> for PyVariableClassObjectBase {
        fn ensure_threadsafe(&self) {}
        fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
            Ok(())
        }
        unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
            tp_dealloc(py, slf, T::type_object_raw(py));
        }
    }

    #[cfg(Py_3_12)]
    pub fn get_contents_ptr<T: PyClassImpl>(
        obj: *mut ffi::PyObject,
    ) -> *mut PyClassObjectContents<T> {
        #[cfg(Py_3_12)]
        {
            let type_obj = unsafe { ffi::Py_TYPE(obj) };
            assert!(!type_obj.is_null());
            let pointer = unsafe { ffi::PyObject_GetTypeData(obj, type_obj) };
            assert!(!pointer.is_null());
            pointer as *mut PyClassObjectContents<T>
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
        ffi,
        impl_::pyclass::{PyClassBaseType, PyClassImpl},
        pycell::PyBorrowError,
        type_object::{PyLayout, PySizedLayout},
        PyTypeInfo, Python,
    };

    use super::{tp_dealloc, PyClassObjectBaseLayout, PyClassObjectContents};

    // The layout of a `PyObject` that uses the static layout
    #[repr(C)]
    pub struct ClassObject<T: PyClassImpl> {
        pub ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
        pub contents: PyClassObjectContents<T>,
    }

    /// Base layout of PyClassObject with a known sized base type.
    /// Corresponds to [PyObject](https://docs.python.org/3/c-api/structures.html#c.PyObject) from the C API.
    #[doc(hidden)]
    #[repr(C)]
    pub struct PyClassObjectBase<T> {
        ob_base: T,
    }

    unsafe impl<T, U> PyLayout<T> for PyClassObjectBase<U> where U: PySizedLayout<T> {}

    impl<T, U> PyClassObjectBaseLayout<T> for PyClassObjectBase<U>
    where
        U: PySizedLayout<T>,
        T: PyTypeInfo,
    {
        fn ensure_threadsafe(&self) {}
        fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
            Ok(())
        }
        unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
            tp_dealloc(py, slf, T::type_object_raw(py));
        }
    }
}

/// Functions for working with `PyObject`s
pub(crate) struct PyObjectLayout {}

impl PyObjectLayout {
    /// Obtain a pointer to the contents of a `PyObject` of type `T`.
    ///
    /// Safety: the provided object must be valid and have the layout indicated by `T`
    pub(crate) unsafe fn get_contents_ptr<T: PyClassImpl>(
        obj: *mut ffi::PyObject,
    ) -> *mut PyClassObjectContents<T> {
        debug_assert!(!obj.is_null());
        if <T::BaseType as PyTypeInfo>::OPAQUE {
            opaque_layout::get_contents_ptr(obj)
        } else {
            let obj: *mut static_layout::ClassObject<T> = obj.cast();
            addr_of_mut!((*obj).contents)
        }
    }

    /// obtain a pointer to the pyclass struct of a `PyObject` of type `T`.
    ///
    /// Safety: the provided object must be valid and have the layout indicated by `T`
    pub(crate) unsafe fn get_data_ptr<T: PyClassImpl>(obj: *mut ffi::PyObject) -> *mut T {
        let contents = PyObjectLayout::get_contents_ptr::<T>(obj);
        (*contents).value.get()
    }

    pub(crate) unsafe fn get_data<T: PyClassImpl>(obj: &ffi::PyObject) -> &T {
        let obj_ref = (obj as *const ffi::PyObject).cast_mut();
        let data_ptr = unsafe { PyObjectLayout::get_data_ptr::<T>(obj_ref) };
        unsafe { &*data_ptr }
    }

    pub(crate) unsafe fn get_borrow_checker<T: PyClassImpl>(
        obj: &ffi::PyObject,
    ) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(obj)
    }

    /// obtain a reference to the data at the start of the `PyObject`.
    ///
    /// Safety: the provided object must be valid and have the layout indicated by `T`
    pub(crate) unsafe fn ob_base<T: PyClassImpl>(
        obj: *mut ffi::PyObject,
    ) -> *mut <T::BaseType as PyClassBaseType>::LayoutAsBase {
        // the base layout is always at the beginning of the `PyObject` so the pointer can simply be casted
        obj as *mut <T::BaseType as PyClassBaseType>::LayoutAsBase
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
            usize_to_py_ssize(std::mem::size_of::<static_layout::ClassObject<T>>())
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
                static_layout::ClassObject<T>,
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
            let offset = memoffset::offset_of!(static_layout::ClassObject<T>, contents)
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
            let offset = memoffset::offset_of!(static_layout::ClassObject<T>, contents)
                + memoffset::offset_of!(PyClassObjectContents<T>, weakref);

            PyObjectOffset::Absolute(usize_to_py_ssize(offset))
        }
    }
}

/// A wrapper around PyObject to provide [PyObjectLayout] functionality.
#[allow(unused)]
pub(crate) struct PyObjectHandle<'a, T: PyClassImpl>(*mut ffi::PyObject, PhantomData<&'a T>);

#[allow(unused)]
impl<'a, T: PyClassImpl> PyObjectHandle<'a, T> {
    /// Safety: obj must point to a valid PyObject with the type `T`
    pub unsafe fn new(obj: *mut ffi::PyObject) -> Self {
        debug_assert!(!obj.is_null());
        PyObjectHandle(obj, PhantomData)
    }

    pub fn contents(&'a self) -> &'a PyClassObjectContents<T> {
        unsafe { &*PyObjectLayout::get_contents_ptr::<T>(self.0) }
    }

    pub fn data(&'a self) -> &'a T {
        unsafe { &*PyObjectLayout::get_data_ptr::<T>(self.0) }
    }

    pub fn ensure_threadsafe(&self) {
        let contents = unsafe { &(*PyObjectLayout::get_contents_ptr::<T>(self.0)) };
        contents.thread_checker.ensure();
        let base = unsafe { &(*PyObjectLayout::ob_base::<T>(self.0)) };
        base.ensure_threadsafe();
    }

    pub fn borrow_checker(&self) {}
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::*;

    use crate::prelude::*;
    use crate::pyclass::boolean_struct::{False, True};

    #[pyclass(crate = "crate", subclass)]
    struct MutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, subclass)]
    struct MutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableBase, frozen, subclass)]
    struct ImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase)]
    struct MutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase)]
    struct MutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfMutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfMutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfMutableBase;

    #[pyclass(crate = "crate", frozen, subclass)]
    struct ImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, subclass)]
    struct MutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableBase, frozen, subclass)]
    struct ImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase)]
    struct MutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase)]
    struct MutableChildOfImmutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = MutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfMutableChildOfImmutableBase;

    #[pyclass(crate = "crate", extends = ImmutableChildOfImmutableBase, frozen)]
    struct ImmutableChildOfImmutableChildOfImmutableBase;

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

    fn assert_mutable<T: PyClass<Frozen = False, PyClassMutability = MutableClass>>() {}
    fn assert_immutable<T: PyClass<Frozen = True, PyClassMutability = ImmutableClass>>() {}
    fn assert_mutable_with_mutable_ancestor<
        T: PyClass<Frozen = False, PyClassMutability = ExtendsMutableAncestor<MutableClass>>,
    >() {
    }
    fn assert_immutable_with_mutable_ancestor<
        T: PyClass<Frozen = True, PyClassMutability = ExtendsMutableAncestor<ImmutableClass>>,
    >() {
    }

    #[test]
    fn test_inherited_mutability() {
        // mutable base
        assert_mutable::<MutableBase>();

        // children of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableBase>();

        // grandchildren of mutable base have a mutable ancestor
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfMutableBase>();
        assert_mutable_with_mutable_ancestor::<MutableChildOfImmutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfMutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfImmutableChildOfMutableBase>();

        // immutable base and children
        assert_immutable::<ImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableBase>();
        assert_immutable::<ImmutableChildOfImmutableChildOfImmutableBase>();

        // mutable children of immutable at any level are simply mutable
        assert_mutable::<MutableChildOfImmutableBase>();
        assert_mutable::<MutableChildOfImmutableChildOfImmutableBase>();

        // children of the mutable child display this property
        assert_mutable_with_mutable_ancestor::<MutableChildOfMutableChildOfImmutableBase>();
        assert_immutable_with_mutable_ancestor::<ImmutableChildOfMutableChildOfImmutableBase>();
    }

    #[test]
    fn test_mutable_borrow_prevents_further_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow_mut();

            // Cannot take any other mutable or immutable borrows whilst the object is borrowed mutably
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all other borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }

    #[test]
    fn test_immutable_borrows_prevent_mutable_borrows() {
        Python::with_gil(|py| {
            let mmm = Py::new(
                py,
                PyClassInitializer::from(MutableBase)
                    .add_subclass(MutableChildOfMutableBase)
                    .add_subclass(MutableChildOfMutableChildOfMutableBase),
            )
            .unwrap();

            let mmm_bound: &Bound<'_, MutableChildOfMutableChildOfMutableBase> = mmm.bind(py);

            let mmm_refmut = mmm_bound.borrow();

            // Further immutable borrows are ok
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRef<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRef<'_, MutableBase>>().is_ok());

            // Further mutable borrows are not ok
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_err());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_err());

            // With the borrow dropped, all mutable borrow attempts will succeed
            drop(mmm_refmut);

            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound
                .extract::<PyRefMut<'_, MutableChildOfMutableBase>>()
                .is_ok());
            assert!(mmm_bound.extract::<PyRefMut<'_, MutableBase>>().is_ok());
        })
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {
            x: u64,
        }

        Python::with_gil(|py| {
            let inst = Py::new(py, MyClass { x: 0 }).unwrap();

            let total_modifications = py.allow_threads(|| {
                std::thread::scope(|s| {
                    // Spawn a bunch of threads all racing to write to
                    // the same instance of `MyClass`.
                    let threads = (0..10)
                        .map(|_| {
                            s.spawn(|| {
                                Python::with_gil(|py| {
                                    // Each thread records its own view of how many writes it made
                                    let mut local_modifications = 0;
                                    for _ in 0..100 {
                                        if let Ok(mut i) = inst.try_borrow_mut(py) {
                                            i.x += 1;
                                            local_modifications += 1;
                                        }
                                    }
                                    local_modifications
                                })
                            })
                        })
                        .collect::<Vec<_>>();

                    // Sum up the total number of writes made by all threads
                    threads.into_iter().map(|t| t.join().unwrap()).sum::<u64>()
                })
            });

            // If the implementation is free of data races, the total number of writes
            // should match the final value of `x`.
            assert_eq!(total_modifications, inst.borrow(py).x);
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_thread_safety_2() {
        struct SyncUnsafeCell<T>(UnsafeCell<T>);
        unsafe impl<T> Sync for SyncUnsafeCell<T> {}

        impl<T> SyncUnsafeCell<T> {
            fn get(&self) -> *mut T {
                self.0.get()
            }
        }

        let data = SyncUnsafeCell(UnsafeCell::new(0));
        let data2 = SyncUnsafeCell(UnsafeCell::new(0));
        let borrow_checker = BorrowChecker(BorrowFlag(AtomicUsize::new(BorrowFlag::UNUSED)));

        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow_mut().is_ok() {
                        // thread 1 writes to both values during the mutable borrow
                        unsafe { *data.get() += 1 };
                        unsafe { *data2.get() += 1 };
                        borrow_checker.release_borrow_mut();
                    }
                }
            });

            s.spawn(|| {
                for _ in 0..1_000_000 {
                    if borrow_checker.try_borrow().is_ok() {
                        // if the borrow checker is working correctly, it should be impossible
                        // for thread 2 to observe a difference in the two values
                        assert_eq!(unsafe { *data.get() }, unsafe { *data2.get() });
                        borrow_checker.release_borrow();
                    }
                }
            });
        });
    }
}
