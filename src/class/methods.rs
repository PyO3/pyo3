// Copyright (c) 2017-present PyO3 Project and Contributors

use std::mem;
use std::ptr;
use std::ffi::CString;

use ::{ffi, exc, class, py_class, PyErr, Python, PyResult, PythonObject};
use objects::PyType;
use function::AbortOnDrop;


#[derive(Copy, Clone)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
}

#[derive(Copy, Clone)]
pub struct PyMethodDef {
    pub ml_name: &'static str,
    pub ml_meth: PyMethodType,
    pub ml_flags: ::c_int,
    pub ml_doc: &'static str,
}

unsafe impl Sync for PyMethodDef {}
unsafe impl Sync for ffi::PyMethodDef {}


pub trait PyClassInit {

    fn init() -> bool;

    fn type_object() -> &'static mut ffi::PyTypeObject;

    fn build_type(py: Python,
                  module_name: Option<&str>,
                  type_object: &mut ffi::PyTypeObject) -> PyResult<PyType>;

}

impl<T> PyClassInit for T where T: PythonObject + py_class::BaseObject {

    default fn init() -> bool { false }

    default fn type_object() -> &'static mut ffi::PyTypeObject {
        static mut TYPE_OBJECT: ffi::PyTypeObject = ffi::PyTypeObject_INIT;
        unsafe {
            &mut TYPE_OBJECT
        }
    }

    default fn build_type(py: Python, module_name: Option<&str>,
                          type_object: &mut ffi::PyTypeObject) -> PyResult<PyType> {
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
        //<T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

        // type size
        type_object.tp_basicsize = <T as py_class::BaseObject>::size() as ffi::Py_ssize_t;

        // buffer protocol
        if let Some(buf) = ffi::PyBufferProcs::new::<T>() {
            static mut BUFFER_PROCS: ffi::PyBufferProcs = ffi::PyBufferProcs_INIT;
            *(unsafe { &mut BUFFER_PROCS }) = buf;
            type_object.tp_as_buffer = unsafe { &mut BUFFER_PROCS };
        } else {
            type_object.tp_as_buffer = 0 as *mut ffi::PyBufferProcs;
        }

        // async methods
        if let Some(buf) = ffi::PyAsyncMethods::new::<T>() {
            static mut ASYNC_METHODS: ffi::PyAsyncMethods = ffi::PyAsyncMethods_INIT;
            *(unsafe { &mut ASYNC_METHODS }) = buf;
            type_object.tp_as_async = unsafe { &mut ASYNC_METHODS };
        } else {
            type_object.tp_as_async = 0 as *mut ffi::PyAsyncMethods;
        }

        // normal methods
        let mut methods = class::methods::py_class_method_defs::<T>();
        if !methods.is_empty() {
            methods.push(ffi::PyMethodDef_INIT);
            type_object.tp_methods = methods.as_ptr() as *mut _;

            static mut METHODS: *const ffi::PyMethodDef = 0 as *const _;
            *(unsafe { &mut METHODS }) = methods.as_ptr();
        }

        unsafe {
            if ffi::PyType_Ready(type_object) == 0 {
                Ok(PyType::from_type_ptr(py, type_object))
            } else {
                Err(PyErr::fetch(py))
            }
        }
    }
}

pub unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where T: py_class::BaseObject
{
    let guard = AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = T::dealloc(py, obj);
    mem::forget(guard);
    r
}

pub fn py_class_method_defs<T>() -> Vec<ffi::PyMethodDef> {
    let mut defs = Vec::new();

    for def in <T as class::context::PyContextProtocolImpl>::py_methods() {
        let meth = match def.ml_meth {
            PyMethodType::PyCFunction(meth) => meth,
            PyMethodType::PyCFunctionWithKeywords(meth) =>
                unsafe {
                    ::std::mem::transmute::<
                            ffi::PyCFunctionWithKeywords, ffi::PyCFunction>(meth)
                }
        };

        let fdef = ffi::PyMethodDef {
            ml_name: CString::new(def.ml_name).expect(
                "Method name must not contain NULL byte").into_raw(),
            ml_meth: Some(meth),
            ml_flags: def.ml_flags,
            ml_doc: 0 as *const ::c_char,
        };
        defs.push(fdef)
    }

    defs
}
