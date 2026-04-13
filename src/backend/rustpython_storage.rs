#![allow(missing_docs)]

use crate::ffi;
use crate::impl_::pycell::{
    GetBorrowChecker, PyClassMutability, PyClassObjectBaseLayout,
};
use crate::impl_::pyclass::{PyClassBaseType, PyClassImpl, PyClassThreadChecker, PyObjectOffset};
use crate::pycell::impl_::{PyClassObjectContents, PyClassObjectLayout};
use crate::pycell::PyBorrowError;
use crate::type_object::PyLayout;
use crate::{PyClass, Python};
use std::any::TypeId;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

struct SidecarEntry {
    ptr: NonNull<()>,
}

unsafe impl Send for SidecarEntry {}

fn sidecar_registry() -> &'static Mutex<HashMap<(usize, TypeId), SidecarEntry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<(usize, TypeId), SidecarEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn ensure_sidecar_slot<T: PyClassImpl>(
    obj: *mut ffi::PyObject,
) -> *mut MaybeUninit<PyClassObjectContents<T>> {
    let key = (obj as usize, TypeId::of::<T>());
    let mut registry = sidecar_registry().lock().unwrap();
    let entry = registry.entry(key).or_insert_with(|| {
        let boxed = Box::new(MaybeUninit::<PyClassObjectContents<T>>::uninit());
        SidecarEntry {
            ptr: NonNull::new(Box::into_raw(boxed).cast::<()>()).expect("box pointer is non-null"),
        }
    });
    entry.ptr.as_ptr().cast()
}

fn get_sidecar_slot<T: PyClassImpl>(obj: *const ffi::PyObject) -> *mut PyClassObjectContents<T> {
    let key = (obj as usize, TypeId::of::<T>());
    let registry = sidecar_registry().lock().unwrap();
    let entry = registry
        .get(&key)
        .expect("missing RustPython sidecar for native pyclass object");
    entry.ptr.as_ptr().cast()
}

fn take_sidecar_slot<T: PyClassImpl>(
    obj: *mut ffi::PyObject,
) -> Option<Box<MaybeUninit<PyClassObjectContents<T>>>> {
    let key = (obj as usize, TypeId::of::<T>());
    sidecar_registry()
        .lock()
        .unwrap()
        .remove(&key)
        .map(|entry| unsafe {
            Box::from_raw(entry.ptr.as_ptr().cast::<MaybeUninit<PyClassObjectContents<T>>>())
        })
}

/// RustPython-native layout for `#[pyclass(extends = <native>)]`.
///
/// RustPython heap types reject CPython-style inline size expansion for native subclasses. This
/// layout keeps the Python object at the base type's size and stores PyO3's class contents in a
/// backend-managed sidecar allocation keyed by object pointer.
#[repr(C)]
pub struct PySidecarClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PySidecarClassObject<T> {}

impl<T: PyClassImpl<Layout = Self>> PyClassObjectLayout<T> for PySidecarClassObject<T> {
    const CONTENTS_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const BASIC_SIZE: ffi::Py_ssize_t = 0;
    const DICT_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const WEAKLIST_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);

    unsafe fn contents_uninit(
        obj: *mut ffi::PyObject,
    ) -> *mut MaybeUninit<PyClassObjectContents<T>> {
        ensure_sidecar_slot::<T>(obj)
    }

    fn contents(&self) -> &PyClassObjectContents<T> {
        unsafe {
            get_sidecar_slot::<T>(self as *const Self as *const ffi::PyObject)
                .as_ref()
                .expect("sidecar contents pointer should be valid")
        }
    }

    fn contents_mut(&mut self) -> &mut PyClassObjectContents<T> {
        unsafe {
            get_sidecar_slot::<T>(self as *mut Self as *mut ffi::PyObject)
                .as_mut()
                .expect("sidecar contents pointer should be valid")
        }
    }

    fn get_ptr(&self) -> *mut T {
        self.contents().value.get()
    }

    fn ob_base(&self) -> &<T::BaseType as PyClassBaseType>::LayoutAsBase {
        &self.ob_base
    }

    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(self)
    }
}

/// RustPython-native layout for `#[pyclass]` types rooted at `object`.
///
/// This still stores PyO3 contents in a sidecar allocation, but reports a semantic
/// `BASIC_SIZE` matching the inline CPython-style class layout so PyO3's frontend
/// invariants and tests remain meaningful.
#[repr(C)]
pub struct PySemanticSidecarClassObject<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    _semantic_contents: MaybeUninit<PyClassObjectContents<T>>,
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PySemanticSidecarClassObject<T> {}
impl<T: PyClass> crate::type_object::PySizedLayout<T> for PySemanticSidecarClassObject<T> {}

impl<T: PyClassImpl<Layout = Self>> PyClassObjectLayout<T> for PySemanticSidecarClassObject<T> {
    const CONTENTS_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const BASIC_SIZE: ffi::Py_ssize_t = {
        let size = core::mem::size_of::<Self>();
        assert!(size <= ffi::Py_ssize_t::MAX as usize);
        size as ffi::Py_ssize_t
    };
    const DICT_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const WEAKLIST_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);

    unsafe fn contents_uninit(
        obj: *mut ffi::PyObject,
    ) -> *mut MaybeUninit<PyClassObjectContents<T>> {
        ensure_sidecar_slot::<T>(obj)
    }

    fn contents(&self) -> &PyClassObjectContents<T> {
        unsafe {
            get_sidecar_slot::<T>(self as *const Self as *const ffi::PyObject)
                .as_ref()
                .expect("sidecar contents pointer should be valid")
        }
    }

    fn contents_mut(&mut self) -> &mut PyClassObjectContents<T> {
        unsafe {
            get_sidecar_slot::<T>(self as *mut Self as *mut ffi::PyObject)
                .as_mut()
                .expect("sidecar contents pointer should be valid")
        }
    }

    fn get_ptr(&self) -> *mut T {
        self.contents().value.get()
    }

    fn ob_base(&self) -> &<T::BaseType as PyClassBaseType>::LayoutAsBase {
        &self.ob_base
    }

    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(self)
    }
}

impl<T: PyClassImpl<Layout = Self>> PyClassObjectBaseLayout<T> for PySemanticSidecarClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents().thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }

    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents().thread_checker.check() {
            return Err(PyBorrowError::new());
        }
        self.ob_base.check_threadsafe()
    }

    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        if let Some(mut sidecar) = take_sidecar_slot::<T>(slf) {
            unsafe {
                sidecar.assume_init_mut().dealloc(py, slf);
                sidecar.assume_init_drop();
            }
        }
        unsafe { <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf) }
    }
}

impl<T: PyClassImpl<Layout = Self>> PyClassObjectBaseLayout<T> for PySidecarClassObject<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents().thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }

    fn check_threadsafe(&self) -> Result<(), PyBorrowError> {
        if !self.contents().thread_checker.check() {
            return Err(PyBorrowError::new());
        }
        self.ob_base.check_threadsafe()
    }

    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        if let Some(mut sidecar) = take_sidecar_slot::<T>(slf) {
            unsafe {
                sidecar.assume_init_mut().dealloc(py, slf);
                sidecar.assume_init_drop();
            }
        }
        unsafe { <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf) }
    }
}
