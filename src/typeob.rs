// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::mem;
use std::ffi::CString;
use std::collections::HashMap;

use {ffi, class};
use err::{PyErr, PyResult};
use python::Python;
use objects::PyType;
use callback::AbortOnDrop;
use class::methods::PyMethodDefType;


/// Basic python type information
/// Implementing this trait for custom struct is enough to make it compatible with
/// python object system
pub trait PyTypeInfo {
    /// Type of objects to store in PyObject struct
    type Type;

    /// Size of the PyObject structure
    fn size() -> usize;

    /// `Type` instance offset inside PyObject structure
    fn offset() -> isize;

    /// Type name
    fn type_name() -> &'static str;

    /// PyTypeObject instance for this type
    fn type_object() -> &'static mut ffi::PyTypeObject;

}

pub trait PyObjectCtor : PyTypeInfo {

    unsafe fn from_ptr(ptr: *mut ffi::PyObject) -> Self;

}


impl<'a, T: ?Sized> PyTypeInfo for &'a T where T: PyTypeInfo {
    type Type = T::Type;

    #[inline]
    default fn size() -> usize {
        <T as PyTypeInfo>::size()
    }

    #[inline]
    default fn offset() -> isize {
        <T as PyTypeInfo>::offset()
    }

    #[inline]
    default fn type_name() -> &'static str {
        <T as PyTypeInfo>::type_name()
    }

    #[inline]
    default fn type_object() -> &'static mut ffi::PyTypeObject {
        <T as PyTypeInfo>::type_object()
    }
}

pub trait PyObjectAlloc {
    type Type;

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using init_val.
    /// `ty` must be derived from the Self type, and the resulting object
    /// must be of type `ty`.
    unsafe fn alloc(py: Python, value: Self::Type) -> PyResult<*mut ffi::PyObject>;

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject);
}

/// A PythonObject that is usable as a base type for #[class]
impl<T> PyObjectAlloc for T where T : PyTypeInfo {
    type Type = T::Type;

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using init_val.
    /// `ty` must be derived from the Self type, and the resulting object
    /// must be of type `ty`.
    unsafe fn alloc(py: Python, value: T::Type) -> PyResult<*mut ffi::PyObject> {
        let _ = <T as PyTypeObject>::type_object(py);

        let obj = ffi::PyType_GenericAlloc(
            <Self as PyTypeInfo>::type_object(), 0);

        let offset = <Self as PyTypeInfo>::offset();
        let ptr = (obj as *mut u8).offset(offset) as *mut Self::Type;
        std::ptr::write(ptr, value);

        Ok(obj)
    }

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(_py: Python, obj: *mut ffi::PyObject) {
        let ptr = (obj as *mut u8).offset(
            <Self as PyTypeInfo>::offset() as isize) as *mut Self::Type;
        std::ptr::drop_in_place(ptr);

        let ty = ffi::Py_TYPE(obj);
        if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del(obj as *mut ::c_void);
        } else {
            ffi::PyObject_Free(obj as *mut ::c_void);
        }
        // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
        // so we need to call DECREF here:
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

/// Trait implemented by Python object types that have a corresponding type object.
pub trait PyTypeObject {

    /// Retrieves the type object for this Python object type.
    fn type_object(py: Python) -> PyType;

}

impl<T> PyTypeObject for T where T: PyObjectAlloc + PyTypeInfo {

    #[inline]
    fn type_object(py: Python) -> PyType {
        let mut ty = <T as PyTypeInfo>::type_object();

        if (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0 {
            unsafe { PyType::from_type_ptr(py, ty) }
        } else {
            // automatically initialize the class on-demand
            initialize_type::<T>(py, None, <T as PyTypeInfo>::type_name(), ty).expect(
                format!("An error occurred while initializing class {}",
                        <T as PyTypeInfo>::type_name()).as_ref());
            unsafe { PyType::from_type_ptr(py, ty) }
        }
    }
}

pub fn initialize_type<T>(py: Python, module_name: Option<&str>, type_name: &str,
                          type_object: &mut ffi::PyTypeObject) -> PyResult<PyType>
    where T: PyObjectAlloc + PyTypeInfo
{
    // type name
    let name = match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, type_name)),
        None => CString::new(type_name)
    };
    let name = name.expect(
        "Module name/type name must not contain NUL byte").into_raw();

    type_object.tp_name = name;

    // dealloc
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // type size
    type_object.tp_basicsize = <T as PyTypeInfo>::size() as ffi::Py_ssize_t;

    // GC support
    <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // descriptor protocol
    <T as class::descr::PyDescrProtocolImpl>::tp_as_descr(type_object);

    // iterator methods
    <T as class::iter::PyIterProtocolImpl>::tp_as_iter(type_object);

    // basic methods
    <T as class::basic::PyObjectProtocolImpl>::tp_as_object(type_object);

    // number methods
    if let Some(meth) = <T as class::number::PyNumberProtocolImpl>::tp_as_number() {
        type_object.tp_as_number = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_number = 0 as *mut ffi::PyNumberMethods;
    }

    // mapping methods
    if let Some(meth) = <T as class::mapping::PyMappingProtocolImpl>::tp_as_mapping() {
        type_object.tp_as_mapping = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_mapping = 0 as *mut ffi::PyMappingMethods;
    }

    // sequence methods
    if let Some(meth) = <T as class::sequence::PySequenceProtocolImpl>::tp_as_sequence() {
        type_object.tp_as_sequence = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_sequence = 0 as *mut ffi::PySequenceMethods;
    }

    // async methods
    if let Some(meth) = <T as class::async::PyAsyncProtocolImpl>::tp_as_async() {
        type_object.tp_as_async = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_async = 0 as *mut ffi::PyAsyncMethods;
    }

    // buffer protocol
    if let Some(meth) = <T as class::buffer::PyBufferProtocolImpl>::tp_as_buffer() {
        type_object.tp_as_buffer = Box::into_raw(Box::new(meth));
    } else {
        type_object.tp_as_buffer = 0 as *mut ffi::PyBufferProcs;
    }

    // normal methods
    let (new, call, mut methods) = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = methods.as_mut_ptr();
        mem::forget(methods);
    }
    // __new__ method
    type_object.tp_new = new;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        type_object.tp_getset = props.as_mut_ptr();
        mem::forget(props);
    }

    // register type object
    unsafe {
        if ffi::PyType_Ready(type_object) == 0 {
            Ok(PyType::from_type_ptr(py, type_object))
        } else {
            Err(PyErr::fetch(py))
        }
    }
}


unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where T: PyTypeInfo
{
    debug!("DEALLOC: {:?}", obj);
    let guard = AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = <T as PyObjectAlloc>::dealloc(py, obj);
    mem::forget(guard);
    r
}

fn py_class_method_defs<T>() -> (Option<ffi::newfunc>,
                                 Option<ffi::PyCFunctionWithKeywords>,
                                 Vec<ffi::PyMethodDef>)  {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = None;

    for def in <T as class::methods::PyMethodsProtocolImpl>::py_methods() {
        match def {
            &PyMethodDefType::New(ref def) => {
                if let class::methods::PyMethodType::PyNewFunc(meth) = def.ml_meth {
                    new = Some(meth)
                }
            },
            &PyMethodDefType::Call(ref def) => {
                if let class::methods::PyMethodType::PyCFunctionWithKeywords(meth) = def.ml_meth {
                    call = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_call slot")
                }
            }
            &PyMethodDefType::Method(ref def) => {
                defs.push(def.as_method_def())
            }
            _ => (),
        }
    }

    for def in <T as class::basic::PyObjectProtocolImpl>::methods() {
        defs.push(def.as_method_def())
    }
    for def in <T as class::async::PyAsyncProtocolImpl>::methods() {
        defs.push(def.as_method_def())
    }
    for def in <T as class::context::PyContextProtocolImpl>::methods() {
        defs.push(def.as_method_def())
    }
    for def in <T as class::mapping::PyMappingProtocolImpl>::methods() {
    defs.push(def.as_method_def())
    }
    for def in <T as class::number::PyNumberProtocolImpl>::methods() {
        defs.push(def.as_method_def())
    }
    for def in <T as class::descr::PyDescrProtocolImpl>::methods() {
        defs.push(def.as_method_def())
    }

    (new, call, defs)
}


fn py_class_properties<T>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = HashMap::new();

    for def in <T as class::methods::PyMethodsProtocolImpl>::py_methods() {
        match def {
            &PyMethodDefType::Getter(ref getter) => {
                let name = getter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).unwrap();
                getter.copy_to(def);
            },
            &PyMethodDefType::Setter(ref setter) => {
                let name = setter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).unwrap();
                setter.copy_to(def);
            },
            _ => (),
        }
    }

    defs.values().map(|i| i.clone()).collect()
}
