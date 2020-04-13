//! `PyClass` trait
use crate::class::methods::{PyMethodDefType, PyMethodsProtocol};
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyLayout};
use crate::{class, ffi, PyCell, PyErr, PyNativeType, PyResult, PyTypeInfo, Python};
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

#[inline]
pub(crate) unsafe fn default_alloc<T: PyTypeInfo>() -> *mut ffi::PyObject {
    let type_obj = T::type_object();
    // if the class derives native types(e.g., PyDict), call special new
    if T::FLAGS & type_flags::EXTENDED != 0 && T::BaseLayout::IS_NATIVE_TYPE {
        let base_tp = <T::BaseType as PyTypeInfo>::type_object();
        if let Some(base_new) = base_tp.tp_new {
            return base_new(type_obj as *const _ as _, ptr::null_mut(), ptr::null_mut());
        }
    }
    let alloc = type_obj.tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
    alloc(type_obj as *const _ as _, 0)
}

/// This trait enables custom alloc/dealloc implementations for `T: PyClass`.
pub trait PyClassAlloc: PyTypeInfo + Sized {
    /// Allocate the actual field for `#[pyclass]`.
    ///
    /// # Safety
    /// This function must return a valid pointer to the Python heap.
    unsafe fn alloc(_py: Python) -> *mut Self::Layout {
        default_alloc::<Self>() as _
    }

    /// Deallocate `#[pyclass]` on the Python heap.
    ///
    /// # Safety
    /// `self_` must be a valid pointer to the Python heap.
    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);
        let obj = self_ as _;
        if ffi::PyObject_CallFinalizerFromDealloc(obj) < 0 {
            return;
        }

        match Self::type_object().tp_free {
            Some(free) => free(obj as *mut c_void),
            None => tp_free_fallback(obj),
        }
    }
}

#[doc(hidden)]
pub unsafe fn tp_free_fallback(obj: *mut ffi::PyObject) {
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
    PyTypeInfo<Layout = PyCell<Self>> + Sized + PyClassAlloc + PyMethodsProtocol
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

    type_object.tp_base = <T::BaseType as PyTypeInfo>::type_object() as *const _ as _;

    type_object.tp_name = match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME))?.into_raw(),
        None => CString::new(T::NAME)?.into_raw(),
    };

    // dealloc
    unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where
        T: PyClassAlloc,
    {
        let pool = crate::GILPool::new();
        let py = pool.python();
        <T as PyClassAlloc>::dealloc(py, (obj as *mut T::Layout) as _)
    }
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // type size
    type_object.tp_basicsize = std::mem::size_of::<T::Layout>() as ffi::Py_ssize_t;

    let mut offset = type_object.tp_basicsize;

    // __dict__ support
    if let Some(dict_offset) = T::Dict::OFFSET {
        offset += dict_offset as ffi::Py_ssize_t;
        type_object.tp_dictoffset = offset;
    }

    // weakref support
    if let Some(weakref_offset) = T::WeakRef::OFFSET {
        offset += weakref_offset as ffi::Py_ssize_t;
        type_object.tp_weaklistoffset = offset;
    }

    // GC support
    <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // descriptor protocol
    <T as class::descr::PyDescrProtocolImpl>::tp_as_descr(type_object);

    // iterator methods
    <T as class::iter::PyIterProtocolImpl>::tp_as_iter(type_object);

    // basic methods
    <T as class::basic::PyObjectProtocolImpl>::tp_as_object(type_object);

    fn to_ptr<T>(value: Option<T>) -> *mut T {
        value
            .map(|v| Box::into_raw(Box::new(v)))
            .unwrap_or_else(ptr::null_mut)
    }

    // number methods
    type_object.tp_as_number = to_ptr(<T as class::number::PyNumberProtocolImpl>::tp_as_number());
    // mapping methods
    type_object.tp_as_mapping =
        to_ptr(<T as class::mapping::PyMappingProtocolImpl>::tp_as_mapping());
    // sequence methods
    type_object.tp_as_sequence =
        to_ptr(<T as class::sequence::PySequenceProtocolImpl>::tp_as_sequence());
    // async methods
    type_object.tp_as_async = to_ptr(<T as class::pyasync::PyAsyncProtocolImpl>::tp_as_async());
    // buffer protocol
    type_object.tp_as_buffer = to_ptr(<T as class::buffer::PyBufferProtocolImpl>::tp_as_buffer());

    // normal methods
    let (new, call, mut methods) = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = Box::into_raw(methods.into_boxed_slice()) as *mut _;
    }

    // __new__ method
    type_object.tp_new = new;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();

    if T::Dict::OFFSET.is_some() {
        props.push(ffi::PyGetSetDef_DICT);
    }
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        type_object.tp_getset = Box::into_raw(props.into_boxed_slice()) as *mut _;
    }

    // set type flags
    py_class_flags::<T>(type_object);

    // register type object
    unsafe {
        if ffi::PyType_Ready(type_object) == 0 {
            Ok(())
        } else {
            PyErr::fetch(py).into()
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

fn py_class_method_defs<T: PyMethodsProtocol>() -> (
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

    for def in <T as class::basic::PyObjectProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::context::PyContextProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::mapping::PyMappingProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::number::PyNumberProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::descr::PyDescrProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }

    py_class_async_methods::<T>(&mut defs);

    (new, call, defs)
}

fn py_class_async_methods<T>(defs: &mut Vec<ffi::PyMethodDef>) {
    for def in <T as class::pyasync::PyAsyncProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
}

fn py_class_properties<T: PyMethodsProtocol>() -> Vec<ffi::PyGetSetDef> {
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
