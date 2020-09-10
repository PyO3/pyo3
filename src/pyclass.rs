//! `PyClass` and related traits.
use crate::class::methods::{PyClassAttributeDef, PyMethodDefType, PyMethods};
use crate::class::proto_methods::PyProtoMethods;
use crate::conversion::{AsPyPointer, FromPyPointer};
use crate::derive_utils::PyBaseTypeUtils;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyLayout};
use crate::types::PyAny;
use crate::{class, ffi, PyCell, PyErr, PyNativeType, PyResult, PyTypeInfo, Python};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::{ptr, thread};

#[inline]
pub(crate) unsafe fn default_new<T: PyTypeInfo>(
    py: Python,
    subtype: *mut ffi::PyTypeObject,
) -> *mut ffi::PyObject {
    // if the class derives native types(e.g., PyDict), call special new
    if T::FLAGS & type_flags::EXTENDED != 0 && T::BaseLayout::IS_NATIVE_TYPE {
        let base_tp = T::BaseType::type_object_raw(py);
        if let Some(base_new) = (*base_tp).tp_new {
            return base_new(subtype, ptr::null_mut(), ptr::null_mut());
        }
    }
    let alloc = (*subtype).tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
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
        if Self::is_exact_instance(obj) && ffi::PyObject_CallFinalizerFromDealloc(obj.as_ptr()) < 0
        {
            // tp_finalize resurrected.
            return;
        }

        match (*ffi::Py_TYPE(obj.as_ptr())).tp_free {
            Some(free) => free(obj.as_ptr() as *mut c_void),
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

#[cfg(not(Py_LIMITED_API))]
pub(crate) fn initialize_type_object<T>(
    py: Python,
    module_name: Option<&str>,
    type_object: &mut ffi::PyTypeObject,
) -> PyResult<()>
where
    T: PyClass,
{
    type_object.tp_doc = match T::DESCRIPTION {
        // PyPy will segfault if passed only a nul terminator as `tp_doc`, ptr::null() is OK though.
        "\0" => ptr::null(),
        s if s.as_bytes().ends_with(b"\0") => s.as_ptr() as _,
        // If the description is not null-terminated, create CString and leak it
        s => CString::new(s)?.into_raw(),
    };

    type_object.tp_base = T::BaseType::type_object_raw(py);

    type_object.tp_name = match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME))?.into_raw(),
        None => CString::new(T::NAME)?.into_raw(),
    };

    // dealloc
    type_object.tp_dealloc = tp_dealloc::<T>();

    // type size
    type_object.tp_basicsize = std::mem::size_of::<T::Layout>() as ffi::Py_ssize_t;

    // __dict__ support
    if let Some(dict_offset) = PyCell::<T>::dict_offset() {
        type_object.tp_dictoffset = dict_offset as ffi::Py_ssize_t;
    }

    // weakref support
    if let Some(weakref_offset) = PyCell::<T>::weakref_offset() {
        type_object.tp_weaklistoffset = weakref_offset as ffi::Py_ssize_t;
    }

    // GC support
    if let Some(gc) = T::gc_methods() {
        unsafe { gc.as_ref() }.update_typeobj(type_object);
    }

    // descriptor protocol
    if let Some(descr) = T::descr_methods() {
        unsafe { descr.as_ref() }.update_typeobj(type_object);
    }

    // iterator methods
    if let Some(iter) = T::iter_methods() {
        unsafe { iter.as_ref() }.update_typeobj(type_object);
    }

    // nb_bool is a part of PyObjectProtocol, but should be placed under tp_as_number
    let mut nb_bool = None;
    // basic methods
    if let Some(basic) = T::basic_methods() {
        unsafe { basic.as_ref() }.update_typeobj(type_object);
        nb_bool = unsafe { basic.as_ref() }.nb_bool;
    }

    // number methods
    type_object.tp_as_number = T::number_methods()
        .map(|mut p| {
            unsafe { p.as_mut() }.nb_bool = nb_bool;
            p.as_ptr()
        })
        .unwrap_or_else(|| nb_bool.map_or_else(ptr::null_mut, ffi::PyNumberMethods::from_nb_bool));
    // mapping methods
    type_object.tp_as_mapping = T::mapping_methods().map_or_else(ptr::null_mut, |p| p.as_ptr());
    // sequence methods
    type_object.tp_as_sequence = T::sequence_methods().map_or_else(ptr::null_mut, |p| p.as_ptr());
    // async methods
    type_object.tp_as_async = T::async_methods().map_or_else(ptr::null_mut, |p| p.as_ptr());
    // buffer protocol
    type_object.tp_as_buffer = T::buffer_methods().map_or_else(ptr::null_mut, |p| p.as_ptr());

    let (new, call, mut methods) = py_class_method_defs::<T>();

    // normal methods
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = Box::into_raw(methods.into_boxed_slice()) as _;
    }

    // __new__ method
    type_object.tp_new = new;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();

    if !T::Dict::IS_DUMMY {
        props.push(ffi::PyGetSetDef_DICT);
    }
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        type_object.tp_getset = Box::into_raw(props.into_boxed_slice()) as _;
    }

    // set type flags
    py_class_flags::<T>(type_object);

    // register type object
    unsafe {
        if ffi::PyType_Ready(type_object) == 0 {
            Ok(())
        } else {
            Err(PyErr::fetch(py))
        }
    }
}

fn py_class_flags<T: PyTypeInfo>(type_object: &mut ffi::PyTypeObject) {
    if type_object.tp_traverse != None
        || type_object.tp_clear != None
        || T::FLAGS & type_flags::GC != 0
    {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC;
    } else {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT;
    }
    if T::FLAGS & type_flags::BASETYPE != 0 {
        type_object.tp_flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
}

pub(crate) fn py_class_attributes<T: PyMethods>() -> impl Iterator<Item = PyClassAttributeDef> {
    T::py_methods().into_iter().filter_map(|def| match def {
        PyMethodDefType::ClassAttribute(attr) => Some(*attr),
        _ => None,
    })
}

fn py_class_method_defs<T: PyMethods>() -> (
    Option<ffi::newfunc>,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = None;

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::New(ref def) => {
                if let class::methods::PyMethodType::PyNewFunc(meth) = def.ml_meth {
                    new = Some(meth)
                }
            }
            PyMethodDefType::Call(ref def) => {
                if let class::methods::PyMethodType::PyCFunctionWithKeywords(meth) = def.ml_meth {
                    call = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_call slot")
                }
            }
            PyMethodDefType::Method(ref def)
            | PyMethodDefType::Class(ref def)
            | PyMethodDefType::Static(ref def) => {
                defs.push(def.as_method_def());
            }
            _ => (),
        }
    }

    (new, call, defs)
}

fn py_class_properties<T: PyMethods>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = std::collections::HashMap::new();

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::Getter(ref getter) => {
                let name = getter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                getter.copy_to(def);
            }
            PyMethodDefType::Setter(ref setter) => {
                let name = setter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                setter.copy_to(def);
            }
            _ => (),
        }
    }

    defs.values().cloned().collect()
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
