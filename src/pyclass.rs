//! `PyClass` and related traits.
use crate::class::methods::{PyClassAttributeDef, PyMethodDefType, PyMethods};
use crate::class::proto_methods::PyProtoMethods;
use crate::conversion::{AsPyPointer, FromPyPointer};
use crate::derive_utils::PyBaseTypeUtils;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyLayout};
use crate::types::PyAny;
use crate::{ffi, PyCell, PyErr, PyNativeType, PyResult, PyTypeInfo, Python};
use std::convert::TryInto;
use std::ffi::CString;
use std::marker::PhantomData;
#[cfg(not(PyPy))]
use std::mem;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::{ptr, thread};

#[cfg(PyPy)]
unsafe fn get_type_alloc(tp: *mut ffi::PyTypeObject) -> Option<ffi::allocfunc> {
    (*tp).tp_alloc
}

#[cfg(not(PyPy))]
unsafe fn get_type_alloc(tp: *mut ffi::PyTypeObject) -> Option<ffi::allocfunc> {
    mem::transmute(ffi::PyType_GetSlot(tp, ffi::Py_tp_alloc))
}

#[cfg(PyPy)]
pub(crate) unsafe fn get_type_free(tp: *mut ffi::PyTypeObject) -> Option<ffi::freefunc> {
    (*tp).tp_free
}

#[cfg(not(PyPy))]
pub(crate) unsafe fn get_type_free(tp: *mut ffi::PyTypeObject) -> Option<ffi::freefunc> {
    mem::transmute(ffi::PyType_GetSlot(tp, ffi::Py_tp_free))
}

#[inline]
pub(crate) unsafe fn default_new<T: PyTypeInfo>(
    py: Python,
    subtype: *mut ffi::PyTypeObject,
) -> *mut ffi::PyObject {
    // if the class derives native types(e.g., PyDict), call special new
    if T::FLAGS & type_flags::EXTENDED != 0 && T::BaseLayout::IS_NATIVE_TYPE {
        #[cfg(not(Py_LIMITED_API))]
        {
            let base_tp = T::BaseType::type_object_raw(py);
            if let Some(base_new) = (*base_tp).tp_new {
                return base_new(subtype, ptr::null_mut(), ptr::null_mut());
            }
        }
        #[cfg(Py_LIMITED_API)]
        {
            // Silence unused parameter warning.
            let _ = py;
            unreachable!("Subclassing native types isn't support in limited API mode");
        }
    }
    let alloc = get_type_alloc(subtype).unwrap_or(ffi::PyType_GenericAlloc);
    alloc(subtype, 0) as _
}

/// This trait enables custom `tp_new`/`tp_dealloc` implementations for `T: PyClass`.
pub trait PyClassAlloc: PyTypeInfo + Sized {
    /// Allocate the actual field for `#[pyclass]`.
    ///
    /// # Safety
    /// This function must return a valid pointer to the Python heap.
    unsafe fn new(py: Python, subtype: *mut ffi::PyTypeObject) -> *mut Self::Layout {
        default_new::<Self>(py, subtype) as _
    }

    /// Deallocate `#[pyclass]` on the Python heap.
    ///
    /// # Safety
    /// `self_` must be a valid pointer to the Python heap.
    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);
        let obj = PyAny::from_borrowed_ptr_or_panic(py, self_ as _);

        match get_type_free(ffi::Py_TYPE(obj.as_ptr())) {
            Some(free) => {
                let ty = ffi::Py_TYPE(obj.as_ptr());
                free(obj.as_ptr() as *mut c_void);
                ffi::Py_DECREF(ty as *mut ffi::PyObject);
            }
            None => tp_free_fallback(obj.as_ptr()),
        }
    }
}

fn tp_dealloc<T: PyClassAlloc>() -> Option<ffi::destructor> {
    unsafe extern "C" fn dealloc<T>(obj: *mut ffi::PyObject)
    where
        T: PyClassAlloc,
    {
        let pool = crate::GILPool::new();
        let py = pool.python();
        <T as PyClassAlloc>::dealloc(py, (obj as *mut T::Layout) as _)
    }
    Some(dealloc::<T>)
}

pub(crate) unsafe fn tp_free_fallback(obj: *mut ffi::PyObject) {
    let ty = ffi::Py_TYPE(obj);
    if ffi::PyType_IS_GC(ty) != 0 {
        ffi::PyObject_GC_Del(obj as *mut c_void);
    } else {
        ffi::PyObject_Free(obj as *mut c_void);
    }

    // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
    // so we need to call DECREF here:
    if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
        ffi::Py_DECREF(ty as *mut ffi::PyObject);
    }
}

/// If `PyClass` is implemented for `T`, then we can use `T` in the Python world,
/// via `PyCell`.
///
/// The `#[pyclass]` attribute automatically implements this trait for your Rust struct,
/// so you don't have to use this trait directly.
pub trait PyClass:
    PyTypeInfo<Layout = PyCell<Self>, AsRefTarget = PyCell<Self>>
    + Sized
    + PyClassSend
    + PyClassAlloc
    + PyMethods
    + PyProtoMethods
{
    /// Specify this class has `#[pyclass(dict)]` or not.
    type Dict: PyClassDict;
    /// Specify this class has `#[pyclass(weakref)]` or not.
    type WeakRef: PyClassWeakRef;
    /// The closest native ancestor. This is `PyAny` by default, and when you declare
    /// `#[pyclass(extends=PyDict)]`, it's `PyDict`.
    type BaseNativeType: PyTypeInfo + PyNativeType;
}

/// For collecting slot items.
#[derive(Default)]
pub(crate) struct TypeSlots(Vec<ffi::PyType_Slot>);

impl TypeSlots {
    fn push(&mut self, slot: c_int, pfunc: *mut c_void) {
        self.0.push(ffi::PyType_Slot { slot, pfunc });
    }
    pub(crate) fn maybe_push(&mut self, slot: c_int, value: Option<*mut c_void>) {
        if let Some(v) = value {
            self.push(slot, v);
        }
    }
}

fn tp_doc<T: PyClass>() -> PyResult<Option<*mut c_void>> {
    Ok(match T::DESCRIPTION {
        "\0" => None,
        s if s.as_bytes().ends_with(b"\0") => Some(s.as_ptr() as _),
        // If the description is not null-terminated, create CString and leak it
        s => Some(CString::new(s)?.into_raw() as _),
    })
}

fn get_type_name<T: PyTypeInfo>(module_name: Option<&str>) -> PyResult<*mut c_char> {
    Ok(match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME))?.into_raw(),
        None => CString::new(T::NAME)?.into_raw(),
    })
}

fn into_raw<T>(vec: Vec<T>) -> *mut c_void {
    Box::into_raw(vec.into_boxed_slice()) as _
}

pub(crate) fn create_type_object<T>(
    py: Python,
    module_name: Option<&str>,
) -> PyResult<*mut ffi::PyTypeObject>
where
    T: PyClass,
{
    let mut slots = TypeSlots::default();

    slots.push(ffi::Py_tp_base, T::BaseType::type_object_raw(py) as _);
    slots.maybe_push(ffi::Py_tp_doc, tp_doc::<T>()?);
    slots.maybe_push(ffi::Py_tp_dealloc, tp_dealloc::<T>().map(|v| v as _));

    let (new, call, methods) = py_class_method_defs::<T>();
    slots.maybe_push(ffi::Py_tp_new, new.map(|v| v as _));
    slots.maybe_push(ffi::Py_tp_call, call.map(|v| v as _));
    // normal methods
    if !methods.is_empty() {
        slots.push(ffi::Py_tp_methods, into_raw(methods));
    }

    // properties
    let props = py_class_properties::<T>();
    if !props.is_empty() {
        slots.push(ffi::Py_tp_getset, into_raw(props));
    }

    // basic methods
    if let Some(basic) = T::basic_methods() {
        unsafe { basic.as_ref() }.update_slots(&mut slots);
    }
    // number methods
    if let Some(number) = T::number_methods() {
        unsafe { number.as_ref() }.update_slots(&mut slots);
    }
    // iterator methods
    if let Some(iter) = T::iter_methods() {
        unsafe { iter.as_ref() }.update_slots(&mut slots);
    }
    // mapping methods
    if let Some(mapping) = T::mapping_methods() {
        unsafe { mapping.as_ref() }.update_slots(&mut slots);
    }
    // sequence methods
    if let Some(sequence) = T::sequence_methods() {
        unsafe { sequence.as_ref() }.update_slots(&mut slots);
    }
    // descriptor protocol
    if let Some(descr) = T::descr_methods() {
        unsafe { descr.as_ref() }.update_slots(&mut slots);
    }
    // async methods
    if let Some(asnc) = T::async_methods() {
        unsafe { asnc.as_ref() }.update_slots(&mut slots);
    }
    // gc methods
    if let Some(gc) = T::gc_methods() {
        unsafe { gc.as_ref() }.update_slots(&mut slots);
    }

    slots.push(0, ptr::null_mut());
    let mut spec = ffi::PyType_Spec {
        name: get_type_name::<T>(module_name)?,
        basicsize: std::mem::size_of::<T::Layout>() as c_int,
        itemsize: 0,
        flags: py_class_flags::<T>(),
        slots: slots.0.as_mut_slice().as_mut_ptr(),
    };

    let type_object = unsafe { ffi::PyType_FromSpec(&mut spec) };
    if type_object.is_null() {
        Err(PyErr::fetch(py))
    } else {
        tp_init_additional::<T>(type_object as _);
        Ok(type_object as _)
    }
}

#[cfg(not(Py_LIMITED_API))]
fn tp_init_additional<T: PyClass>(type_object: *mut ffi::PyTypeObject) {
    // Just patch the type objects for the things there's no
    // PyType_FromSpec API for... there's no reason this should work,
    // except for that it does and we have tests.

    // Running this causes PyPy to segfault.
    #[cfg(all(not(PyPy), not(Py_3_10)))]
    if T::DESCRIPTION != "\0" {
        unsafe {
            // Until CPython 3.10, tp_doc was treated specially for heap-types,
            // and it removed the text_signature value from it. We go in after
            // the fact and replace tp_doc with something that _does_ include
            // the text_signature value!
            ffi::PyObject_Free((*type_object).tp_doc as _);
            let data = ffi::PyObject_Malloc(T::DESCRIPTION.len());
            data.copy_from(T::DESCRIPTION.as_ptr() as _, T::DESCRIPTION.len());
            (*type_object).tp_doc = data as _;
        }
    }

    if let Some(buffer) = T::buffer_methods() {
        unsafe {
            (*(*type_object).tp_as_buffer).bf_getbuffer = buffer.as_ref().bf_getbuffer;
            (*(*type_object).tp_as_buffer).bf_releasebuffer = buffer.as_ref().bf_releasebuffer;
        }
    }
    // __dict__ support
    if let Some(dict_offset) = PyCell::<T>::dict_offset() {
        unsafe {
            (*type_object).tp_dictoffset = dict_offset as ffi::Py_ssize_t;
        }
    }
    // weakref support
    if let Some(weakref_offset) = PyCell::<T>::weakref_offset() {
        unsafe {
            (*type_object).tp_weaklistoffset = weakref_offset as ffi::Py_ssize_t;
        }
    }
}

#[cfg(Py_LIMITED_API)]
fn tp_init_additional<T: PyClass>(_type_object: *mut ffi::PyTypeObject) {}

fn py_class_flags<T: PyClass + PyTypeInfo>() -> c_uint {
    let mut flags = if T::gc_methods().is_some() || T::FLAGS & type_flags::GC != 0 {
        ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC
    } else {
        ffi::Py_TPFLAGS_DEFAULT
    };
    if T::FLAGS & type_flags::BASETYPE != 0 {
        flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
    flags.try_into().unwrap()
}

pub(crate) fn py_class_attributes<T: PyMethods>() -> impl Iterator<Item = PyClassAttributeDef> {
    T::py_methods().into_iter().filter_map(|def| match def {
        PyMethodDefType::ClassAttribute(attr) => Some(attr.to_owned()),
        _ => None,
    })
}

fn fallback_new() -> Option<ffi::newfunc> {
    unsafe extern "C" fn fallback_new(
        _subtype: *mut ffi::PyTypeObject,
        _args: *mut ffi::PyObject,
        _kwds: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject {
        crate::callback_body!(py, {
            Err::<(), _>(crate::exceptions::PyTypeError::new_err(
                "No constructor defined",
            ))
        })
    }
    Some(fallback_new)
}

fn py_class_method_defs<T: PyMethods>() -> (
    Option<ffi::newfunc>,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = fallback_new();

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::New(ref def) => {
                new = def.get_new_func();
                debug_assert!(new.is_some());
            }
            PyMethodDefType::Call(ref def) => {
                call = def.get_cfunction_with_keywords();
                debug_assert!(call.is_some());
            }
            PyMethodDefType::Method(ref def)
            | PyMethodDefType::Class(ref def)
            | PyMethodDefType::Static(ref def) => {
                defs.push(def.as_method_def());
            }
            _ => (),
        }
    }

    if !defs.is_empty() {
        defs.push(ffi::PyMethodDef_INIT);
    }

    (new, call, defs)
}

fn py_class_properties<T: PyClass>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = std::collections::HashMap::new();

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::Getter(ref getter) => {
                if !defs.contains_key(getter.name) {
                    let _ = defs.insert(getter.name.to_owned(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(getter.name).expect("Failed to call get_mut");
                getter.copy_to(def);
            }
            PyMethodDefType::Setter(ref setter) => {
                if !defs.contains_key(setter.name) {
                    let _ = defs.insert(setter.name.to_owned(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(setter.name).expect("Failed to call get_mut");
                setter.copy_to(def);
            }
            _ => (),
        }
    }

    let mut props: Vec<_> = defs.values().cloned().collect();
    if !T::Dict::IS_DUMMY {
        props.push(ffi::PyGetSetDef_DICT);
    }
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
    }
    props
}

/// This trait is implemented for `#[pyclass]` and handles following two situations:
/// 1. In case `T` is `Send`, stub `ThreadChecker` is used and does nothing.
///    This implementation is used by default. Compile fails if `T: !Send`.
/// 2. In case `T` is `!Send`, `ThreadChecker` panics when `T` is accessed by another thread.
///    This implementation is used when `#[pyclass(unsendable)]` is given.
///    Panicking makes it safe to expose `T: !Send` to the Python interpreter, where all objects
///    can be accessed by multiple threads by `threading` module.
pub trait PyClassSend: Sized {
    type ThreadChecker: PyClassThreadChecker<Self>;
}

#[doc(hidden)]
pub trait PyClassThreadChecker<T>: Sized {
    fn ensure(&self);
    fn new() -> Self;
    private_decl! {}
}

/// Stub checker for `Send` types.
#[doc(hidden)]
pub struct ThreadCheckerStub<T: Send>(PhantomData<T>);

impl<T: Send> PyClassThreadChecker<T> for ThreadCheckerStub<T> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

impl<T: PyNativeType> PyClassThreadChecker<T> for ThreadCheckerStub<crate::PyObject> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

/// Thread checker for unsendable types.
/// Panics when the value is accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerImpl<T>(thread::ThreadId, PhantomData<T>);

impl<T> PyClassThreadChecker<T> for ThreadCheckerImpl<T> {
    fn ensure(&self) {
        if thread::current().id() != self.0 {
            panic!(
                "{} is unsendable, but sent to another thread!",
                std::any::type_name::<T>()
            );
        }
    }
    fn new() -> Self {
        ThreadCheckerImpl(thread::current().id(), PhantomData)
    }
    private_impl! {}
}

/// Thread checker for types that have `Send` and `extends=...`.
/// Ensures that `T: Send` and the parent is not accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerInherited<T: Send, U: PyBaseTypeUtils>(PhantomData<T>, U::ThreadChecker);

impl<T: Send, U: PyBaseTypeUtils> PyClassThreadChecker<T> for ThreadCheckerInherited<T, U> {
    fn ensure(&self) {
        self.1.ensure();
    }
    fn new() -> Self {
        ThreadCheckerInherited(PhantomData, U::ThreadChecker::new())
    }
    private_impl! {}
}
