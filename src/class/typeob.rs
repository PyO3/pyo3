// Copyright (c) 2017-present PyO3 Project and Contributors

use std::mem;
use std::ffi::CString;
use std::collections::HashMap;

use ::{ffi, class, PyErr, Python, PyResult, PythonObject};
use objects::{PyType, PyModule};
use callback::AbortOnDrop;
use class::{BaseObject, PyMethodDefType};


pub trait PyTypeObject : BaseObject + PythonObject {

    fn add_to_module(py: Python, module: &PyModule) -> PyResult<()>;

    unsafe fn type_obj() -> &'static mut ffi::PyTypeObject;

    unsafe fn initialized(py: Python, module_name: Option<&str>) -> PyType;

}

pub fn initialize_type<T>(py: Python, module_name: Option<&str>,
                          type_object: &mut ffi::PyTypeObject) -> PyResult<PyType>
    where T: BaseObject + PythonObject
{
    // type name
    let name = match module_name {
        Some(module_name) => CString::new(
            format!("{}.{}", module_name, stringify!(type_name))),
        None => CString::new(stringify!(type_name))
    };
    let name = name.expect(
        "Module name/type name must not contain NUL byte").into_raw();

    type_object.tp_name = name;

    // dealloc
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // GC support
    <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // type size
    type_object.tp_basicsize = <T as BaseObject>::size() as ffi::Py_ssize_t;

    // descriptor protocol
    type_object.tp_descr_get = class::descr::get_descrfunc::<T>();
    type_object.tp_descr_set = class::descr::set_descrfunc::<T>();

    // number methods
    if let Some(meth) = ffi::PyNumberMethods::new::<T>() {
        static mut NB_METHODS: ffi::PyNumberMethods = ffi::PyNumberMethods_INIT;
        *(unsafe { &mut NB_METHODS }) = meth;
        type_object.tp_as_number = unsafe { &mut NB_METHODS };
    } else {
        type_object.tp_as_number = 0 as *mut ffi::PyNumberMethods;
    }

    // mapping methods
    if let Some(meth) = ffi::PyMappingMethods::new::<T>() {
        static mut MP_METHODS: ffi::PyMappingMethods = ffi::PyMappingMethods_INIT;
        *(unsafe { &mut MP_METHODS }) = meth;
        type_object.tp_as_mapping = unsafe { &mut MP_METHODS };
    } else {
        type_object.tp_as_mapping = 0 as *mut ffi::PyMappingMethods;
    }

    // sequence methods
    if let Some(meth) = ffi::PySequenceMethods::new::<T>() {
        static mut SQ_METHODS: ffi::PySequenceMethods = ffi::PySequenceMethods_INIT;
        *(unsafe { &mut SQ_METHODS }) = meth;
        type_object.tp_as_sequence = unsafe { &mut SQ_METHODS };
    } else {
        type_object.tp_as_sequence = 0 as *mut ffi::PySequenceMethods;
    }

    // async methods
    if let Some(meth) = ffi::PyAsyncMethods::new::<T>() {
        static mut ASYNC_METHODS: ffi::PyAsyncMethods = ffi::PyAsyncMethods_INIT;
        *(unsafe { &mut ASYNC_METHODS }) = meth;
        type_object.tp_as_async = unsafe { &mut ASYNC_METHODS };
    } else {
        type_object.tp_as_async = 0 as *mut ffi::PyAsyncMethods;
    }

    // buffer protocol
    if let Some(meth) = ffi::PyBufferProcs::new::<T>() {
        static mut BUFFER_PROCS: ffi::PyBufferProcs = ffi::PyBufferProcs_INIT;
        *(unsafe { &mut BUFFER_PROCS }) = meth;
        type_object.tp_as_buffer = unsafe { &mut BUFFER_PROCS };
    } else {
        type_object.tp_as_buffer = 0 as *mut ffi::PyBufferProcs;
    }

    // normal methods
    let mut methods = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = methods.as_ptr() as *mut _;

        static mut METHODS: *const ffi::PyMethodDef = 0 as *const _;
        *(unsafe { &mut METHODS }) = methods.as_ptr();
    }

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

unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject) where T: BaseObject
{
    let guard = AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = T::dealloc(py, obj);
    mem::forget(guard);
    r
}

fn py_class_method_defs<T>() -> Vec<ffi::PyMethodDef> {
    let mut defs = Vec::new();

    for def in <T as class::context::PyContextProtocolImpl>::py_methods() {
        match def {
            &PyMethodDefType::Method(ref def) => defs.push(def.as_method_def()),
            _ => (),
        }
    }

    for def in <T as class::number::PyNumberProtocolImpl>::py_methods() {
        match def {
            &PyMethodDefType::Method(ref def) => defs.push(def.as_method_def()),
            _ => (),
        }
    }
    for def in <T as class::methods::PyMethodsProtocolImpl>::py_methods() {
        match def {
            &PyMethodDefType::Method(ref def) => defs.push(def.as_method_def()),
            _ => (),
        }
    }

    defs
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
