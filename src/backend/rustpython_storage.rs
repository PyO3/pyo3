#![allow(missing_docs)]

use crate::ffi::{self, SidecarCleanup};
use crate::impl_::pycell::{
    GetBorrowChecker, PyClassMutability, PyClassObjectBaseLayout,
};
use crate::impl_::pyclass::{PyClassBaseType, PyClassImpl, PyClassThreadChecker, PyObjectOffset};
use crate::internal::get_slot::{TP_CLEAR, TP_TRAVERSE};
use crate::pycell::impl_::{PyClassObjectContents, PyClassObjectLayout};
use crate::pycell::PyBorrowError;
use crate::type_object::PyLayout;
use crate::{PyClass, PyTypeInfo, Python};
use std::any::TypeId;
use std::collections::HashMap;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

trait SemanticBaseInlineSize {
    const BASIC_SIZE: usize;
}

impl SemanticBaseInlineSize for crate::types::PyAny {
    const BASIC_SIZE: usize = mem::size_of::<crate::impl_::pycell::PyClassObjectBase<ffi::PyObject>>();
}

impl<T> SemanticBaseInlineSize for T
where
    T: PyClassImpl + PyTypeInfo,
    T::Layout: PyClassObjectLayout<T>,
{
    const BASIC_SIZE: usize = <T::Layout as PyClassObjectLayout<T>>::BASIC_SIZE as usize;
}

const fn semantic_inline_size<T>() -> usize
where
    T: PyClassImpl,
    T::BaseType: SemanticBaseInlineSize,
{
    let base_size = <T::BaseType as SemanticBaseInlineSize>::BASIC_SIZE;
    let contents_align = mem::align_of::<PyClassObjectContents<T>>();
    let contents_size = mem::size_of::<PyClassObjectContents<T>>();
    align_up(base_size, contents_align) + contents_size
}

struct SidecarEntry {
    ptr: NonNull<()>,
    cleanup: SidecarCleanup,
    tp_traverse: Option<ffi::traverseproc>,
    tp_clear: Option<ffi::inquiry>,
}

unsafe impl Send for SidecarEntry {}

fn sidecar_registry() -> &'static Mutex<HashMap<(usize, TypeId), SidecarEntry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<(usize, TypeId), SidecarEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn ensure_sidecar_slot<T: PyClassImpl + PyTypeInfo>(
    obj: *mut ffi::PyObject,
) -> *mut MaybeUninit<PyClassObjectContents<T>> {
    let key = (obj as usize, TypeId::of::<T>());
    let mut registry = sidecar_registry().lock().unwrap();
    let entry = registry.entry(key).or_insert_with(|| {
        let py = unsafe { Python::assume_attached() };
        let boxed = Box::new(MaybeUninit::<PyClassObjectContents<T>>::uninit());
        SidecarEntry {
            ptr: NonNull::new(Box::into_raw(boxed).cast::<()>()).expect("box pointer is non-null"),
            cleanup: cleanup_sidecar_entry::<T>,
            tp_traverse: T::type_object(py).get_slot(TP_TRAVERSE),
            tp_clear: T::type_object(py).get_slot(TP_CLEAR),
        }
    });
    entry.ptr.as_ptr().cast()
}

fn get_sidecar_slot<T: PyClassImpl + PyTypeInfo>(obj: *const ffi::PyObject) -> *mut PyClassObjectContents<T> {
    let key = (obj as usize, TypeId::of::<T>());
    let registry = sidecar_registry().lock().unwrap();
    let entry = registry
        .get(&key)
        .expect("missing RustPython sidecar for native pyclass object");
    entry.ptr.as_ptr().cast()
}

fn owner_registry() -> &'static Mutex<HashMap<usize, bool>> {
    static REGISTRY: OnceLock<Mutex<HashMap<usize, bool>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe extern "C" fn cleanup_sidecar_entry<T: PyClassImpl + PyTypeInfo>(
    owner: *mut ffi::PyObject,
    sidecar: *mut std::ffi::c_void,
)
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    let py = unsafe { Python::assume_attached() };
    let ptr = NonNull::new(sidecar.cast::<MaybeUninit<PyClassObjectContents<T>>>())
        .expect("sidecar pointer must be non-null");
    let sidecar = unsafe { Box::from_raw(ptr.as_ptr()) };
    let mut sidecar = sidecar;
    unsafe {
        sidecar.assume_init_mut().dealloc(py, owner);
        sidecar.assume_init_drop();
    }
}

unsafe extern "C" fn cleanup_all_sidecars(owner: *mut ffi::PyObject, _marker: *mut std::ffi::c_void) {
    let owner_key = owner as usize;
    owner_registry().lock().unwrap().remove(&owner_key);

    let entries = {
        let mut registry = sidecar_registry().lock().unwrap();
        let keys = registry
            .keys()
            .filter(|(obj, _)| *obj == owner_key)
            .copied()
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|key| registry.remove(&key))
            .collect::<Vec<_>>()
    };

    for entry in entries {
        unsafe { (entry.cleanup)(owner, entry.ptr.as_ptr().cast::<std::ffi::c_void>()) };
    }
}

pub(crate) unsafe extern "C" fn traverse_sidecars(
    owner: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut std::ffi::c_void,
) -> std::os::raw::c_int {
    let owner_key = owner as usize;
    let callbacks = {
        let registry = sidecar_registry().lock().unwrap();
        registry
            .iter()
            .filter_map(|((obj, _), entry)| (*obj == owner_key).then_some(entry.tp_traverse))
            .collect::<Vec<_>>()
    };
    let mut rc = 0;
    for callback in callbacks {
        if let Some(callback) = callback {
            rc = unsafe { callback(owner, visit, arg) };
            if rc != 0 {
                break;
            }
        }
    }
    rc
}

pub(crate) unsafe extern "C" fn clear_sidecars(
    owner: *mut ffi::PyObject,
    _visit: ffi::visitproc,
    _arg: *mut std::ffi::c_void,
) {
    let _ = owner;
}

pub(crate) fn install_sidecar_owner<T: PyClassImpl + PyTypeInfo>(_py: Python<'_>, obj: *mut ffi::PyObject)
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyClassObjectBaseLayout<T::BaseType>,
{
    let obj_key = obj as usize;
    let mut owners = owner_registry().lock().unwrap();
    if owners.insert(obj_key, true).is_none() {
        let rc = unsafe {
            ffi::PyRustPython_InstallSidecarOwner(
                obj,
                obj.cast::<std::ffi::c_void>(),
                cleanup_all_sidecars,
                traverse_sidecars,
                clear_sidecars,
            )
        };
        assert_eq!(rc, 0, "failed to install RustPython sidecar owner");
    }
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

impl<T: PyClassImpl<Layout = Self> + PyTypeInfo> PyClassObjectLayout<T> for PySidecarClassObject<T> {
    const CONTENTS_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const HAS_EMBEDDED_CONTENTS: bool = false;
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
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PySemanticSidecarClassObject<T> {}
impl<T: PyClass> crate::type_object::PySizedLayout<T> for PySemanticSidecarClassObject<T> {}

impl<T> PyClassObjectLayout<T> for PySemanticSidecarClassObject<T>
where
    T: PyClassImpl<Layout = Self> + PyTypeInfo,
    T::BaseType: SemanticBaseInlineSize,
{
    const CONTENTS_OFFSET: PyObjectOffset = PyObjectOffset::Absolute(0);
    const HAS_EMBEDDED_CONTENTS: bool = false;
    const BASIC_SIZE: ffi::Py_ssize_t = {
        let size = semantic_inline_size::<T>();
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

impl<T> PyClassObjectBaseLayout<T> for PySemanticSidecarClassObject<T>
where
    T: PyClassImpl<Layout = Self> + PyTypeInfo,
    T::BaseType: SemanticBaseInlineSize,
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
        let _ = (py, slf);
        unsafe { <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf) }
    }
}

impl<T: PyClassImpl<Layout = Self> + PyTypeInfo> PyClassObjectBaseLayout<T> for PySidecarClassObject<T>
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
        let _ = (py, slf);
        unsafe { <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf) }
    }
}
