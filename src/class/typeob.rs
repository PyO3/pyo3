// Copyright (c) 2017-present PyO3 Project and Contributors

use std::mem;
use std::ffi::CString;
use std::collections::HashMap;

use ::{ffi, class, PyErr, Python, PyResult, Py};
use objects::PyType;
use callback::AbortOnDrop;
use class::{BaseObject, PyMethodDefType};


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


impl<'a, T> PyTypeInfo for Py<'a, T> where T: PyTypeInfo {
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


/// Trait implemented by Python object types that have a corresponding type object.
pub trait PyTypeObject {

    /// Retrieves the type object for this Python object type.
    fn type_object(py: Python) -> PyType;

}

impl<T> PyTypeObject for T where T: BaseObject + PyTypeInfo {

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
    where T: BaseObject + PyTypeInfo
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
    // <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // descriptor protocol
    // <T as class::descr::PyDescrProtocolImpl>::tp_as_descr(type_object);

    // iterator methods
    // <T as class::iter::PyIterProtocolImpl>::tp_as_iter(type_object);

    // basic methods
    <T as class::basic::PyObjectProtocolImpl>::tp_as_object(type_object);

    // number methods
    /*if let Some(meth) = <T as class::number::PyNumberProtocolImpl>::tp_as_number() {
        static mut NB_METHODS: ffi::PyNumberMethods = ffi::PyNumberMethods_INIT;
        *(unsafe { &mut NB_METHODS }) = meth;
        type_object.tp_as_number = unsafe { &mut NB_METHODS };
        mem::forget(meth);
    } else {
        type_object.tp_as_number = 0 as *mut ffi::PyNumberMethods;
    }

    // mapping methods
    if let Some(meth) = <T as class::mapping::PyMappingProtocolImpl>::tp_as_mapping() {
        static mut MP_METHODS: ffi::PyMappingMethods = ffi::PyMappingMethods_INIT;
        *(unsafe { &mut MP_METHODS }) = meth;
        type_object.tp_as_mapping = unsafe { &mut MP_METHODS };
        mem::forget(meth);
    } else {
        type_object.tp_as_mapping = 0 as *mut ffi::PyMappingMethods;
    }

    // sequence methods
    if let Some(meth) = <T as class::sequence::PySequenceProtocolImpl>::tp_as_sequence() {
        static mut SQ_METHODS: ffi::PySequenceMethods = ffi::PySequenceMethods_INIT;
        *(unsafe { &mut SQ_METHODS }) = meth;
        type_object.tp_as_sequence = unsafe { &mut SQ_METHODS };
        mem::forget(meth);
    } else {
        type_object.tp_as_sequence = 0 as *mut ffi::PySequenceMethods;
    }

    // async methods
    if let Some(meth) = <T as class::async::PyAsyncProtocolImpl>::tp_as_async() {
        static mut ASYNC_METHODS: ffi::PyAsyncMethods = ffi::PyAsyncMethods_INIT;
        *(unsafe { &mut ASYNC_METHODS }) = meth;
        type_object.tp_as_async = unsafe { &mut ASYNC_METHODS };
        mem::forget(meth);
    } else {
        type_object.tp_as_async = 0 as *mut ffi::PyAsyncMethods;
    }

    // buffer protocol
    if let Some(meth) = ffi::PyBufferProcs::new::<T>() {
        static mut BUFFER_PROCS: ffi::PyBufferProcs = ffi::PyBufferProcs_INIT;
        *(unsafe { &mut BUFFER_PROCS }) = meth;
        type_object.tp_as_buffer = unsafe { &mut BUFFER_PROCS };
        mem::forget(meth);
    } else {
        type_object.tp_as_buffer = 0 as *mut ffi::PyBufferProcs;
    }

    // normal methods
    let (new, call, mut methods) = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = methods.as_ptr() as *mut _;

        static mut METHODS: *const ffi::PyMethodDef = 0 as *const _;
        *(unsafe { &mut METHODS }) = methods.as_ptr();

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
        let props = props.into_boxed_slice();
        type_object.tp_getset = props.as_ptr() as *mut _;

        static mut PROPS: *const ffi::PyGetSetDef = 0 as *const _;
        *(unsafe { &mut PROPS }) = props.as_ptr();

        // strange
        mem::forget(props);
    }*/

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
    let guard = AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = <T as BaseObject>::dealloc(&py, obj);
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
