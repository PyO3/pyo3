use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pytypedefs::PyThreadState;
use crate::rustpython_runtime;
use rustpython_vm::scope::Scope;
use std::ffi::{c_char, c_int, c_void};
use crate::PyObject_Call;

#[inline]
pub unsafe fn PyEval_EvalCode(
    arg1: *mut PyObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject {
    if arg1.is_null() || arg2.is_null() {
        return std::ptr::null_mut();
    }
    let code = ptr_to_pyobject_ref_borrowed(arg1);
    let globals = ptr_to_pyobject_ref_borrowed(arg2);
    let locals = if arg3.is_null() {
        None
    } else {
        Some(ptr_to_pyobject_ref_borrowed(arg3))
    };
    rustpython_runtime::with_vm(|vm| {
        let Ok(code) = code.downcast::<rustpython_vm::builtins::PyCode>() else {
            return std::ptr::null_mut();
        };
        let Ok(globals) = globals.downcast::<rustpython_vm::builtins::PyDict>() else {
            return std::ptr::null_mut();
        };
        let locals = locals
            .and_then(|o| o.downcast::<rustpython_vm::builtins::PyDict>().ok())
            .map(rustpython_vm::function::ArgMapping::from_dict_exact);
        let scope = Scope::with_builtins(locals, globals, vm);
        vm.run_code_obj(code, scope)
            .map(pyobject_ref_to_ptr)
            .unwrap_or_else(|exc| {
                set_vm_exception(exc);
                std::ptr::null_mut()
            })
    })
}

#[inline]
pub unsafe fn PyEval_EvalCodeEx(
    co: *mut PyObject,
    globals: *mut PyObject,
    locals: *mut PyObject,
    _args: *const *mut PyObject,
    _argc: c_int,
    _kwds: *const *mut PyObject,
    _kwdc: c_int,
    _defs: *const *mut PyObject,
    _defc: c_int,
    _kwdefs: *mut PyObject,
    _closure: *mut PyObject,
) -> *mut PyObject {
    PyEval_EvalCode(co, globals, locals)
}

#[cfg(not(Py_3_13))]
#[inline]
pub unsafe fn PyEval_CallObjectWithKeywords(
    func: *mut PyObject,
    obj: *mut PyObject,
    kwargs: *mut PyObject,
) -> *mut PyObject {
    PyObject_Call(func, obj, kwargs)
}

#[cfg(not(Py_3_13))]
unsafe extern "C" {
    pub fn PyEval_CallFunction(_obj: *mut PyObject, _format: *const c_char, ...) -> *mut PyObject;
    pub fn PyEval_CallMethod(
        _obj: *mut PyObject,
        _methodname: *const c_char,
        _format: *const c_char,
        ...
    ) -> *mut PyObject;
}

#[inline]
pub unsafe fn PyEval_GetBuiltins() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.builtins.dict().into()))
}

#[inline]
pub unsafe fn PyEval_GetGlobals() -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyEval_GetLocals() -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyEval_GetFrame() -> *mut crate::PyFrameObject {
    std::ptr::null_mut()
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyEval_GetFrameBuiltins() -> *mut PyObject {
    PyEval_GetBuiltins()
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyEval_GetFrameGlobals() -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyEval_GetFrameLocals() -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_AddPendingCall(
    _func: Option<extern "C" fn(arg1: *mut c_void) -> c_int>,
    _arg: *mut c_void,
) -> c_int {
    0
}

#[inline]
pub unsafe fn Py_MakePendingCalls() -> c_int {
    0
}

#[inline]
pub unsafe fn Py_SetRecursionLimit(_arg1: c_int) {}

#[inline]
pub unsafe fn Py_GetRecursionLimit() -> c_int {
    1000
}

#[cfg(Py_3_9)]
#[inline]
pub unsafe fn Py_EnterRecursiveCall(_arg1: *const c_char) -> c_int {
    0
}

#[cfg(Py_3_9)]
#[inline]
pub unsafe fn Py_LeaveRecursiveCall() {}

#[inline]
pub unsafe fn PyEval_GetFuncName(_arg1: *mut PyObject) -> *const c_char {
    std::ptr::null()
}

#[inline]
pub unsafe fn PyEval_GetFuncDesc(_arg1: *mut PyObject) -> *const c_char {
    c"()".as_ptr()
}

#[inline]
pub unsafe fn PyEval_EvalFrame(_arg1: *mut crate::PyFrameObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyEval_EvalFrameEx(_f: *mut crate::PyFrameObject, _exc: c_int) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyEval_SaveThread() -> *mut PyThreadState {
    crate::PyThreadState_Get()
}

#[inline]
pub unsafe fn PyEval_RestoreThread(_arg1: *mut PyThreadState) {}

#[cfg(not(Py_3_13))]
#[inline]
pub unsafe fn PyEval_ThreadsInitialized() -> c_int {
    1
}

#[inline]
pub unsafe fn PyEval_InitThreads() {}

#[cfg(not(Py_3_13))]
#[inline]
pub unsafe fn PyEval_AcquireLock() {}

#[cfg(not(Py_3_13))]
#[inline]
pub unsafe fn PyEval_ReleaseLock() {}

#[inline]
pub unsafe fn PyEval_AcquireThread(_tstate: *mut PyThreadState) {}

#[inline]
pub unsafe fn PyEval_ReleaseThread(_tstate: *mut PyThreadState) {}
