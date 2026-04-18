use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyBaseException, PyBaseExceptionRef, PyTuple, PyType};
use rustpython_vm::exceptions::ExceptionCtor;
use rustpython_vm::{AsObject, PyObjectRef, TryFromObject};
use std::ffi::{c_char, c_int, CStr};
use std::sync::{Mutex, OnceLock};

opaque_struct!(pub PyBaseExceptionObject);
opaque_struct!(pub PyStopIterationObject);
opaque_struct!(pub PyOSErrorObject);
opaque_struct!(pub PySyntaxErrorObject);
opaque_struct!(pub PySystemExitObject);
opaque_struct!(pub PyUnicodeErrorObject);

fn current_exception_slot() -> &'static Mutex<Option<usize>> {
    static CURRENT_EXCEPTION: OnceLock<Mutex<Option<usize>>> = OnceLock::new();
    CURRENT_EXCEPTION.get_or_init(|| Mutex::new(None))
}

macro_rules! exc_statics {
    ($($name:ident => $py:literal),+ $(,)?) => {
        $(
            #[allow(non_upper_case_globals)]
            pub static mut $name: *mut PyObject = std::ptr::null_mut();
        )+

        pub(crate) fn init_exception_symbols(vm: &rustpython_vm::VirtualMachine) {
            unsafe {
                $(
                    if $name.is_null() {
                        if let Ok(obj) = vm.builtins.get_attr($py, vm) {
                            $name = pyobject_ref_to_ptr(obj);
                        }
                    }
                )+
                if PyExc_EnvironmentError.is_null() {
                    PyExc_EnvironmentError = PyExc_OSError;
                }
                if PyExc_IOError.is_null() {
                    PyExc_IOError = PyExc_OSError;
                }
                if PyExc_WindowsError.is_null() {
                    PyExc_WindowsError = PyExc_OSError;
                }
                if PyExc_RecursionErrorInst.is_null() && !PyExc_RecursionError.is_null() {
                    if let Ok(exc) = vm.invoke_exception(
                        ptr_to_pyobject_ref_borrowed(PyExc_RecursionError)
                            .downcast::<PyType>()
                            .expect("RecursionError should be a type"),
                        vec![],
                    ) {
                        PyExc_RecursionErrorInst = pyobject_ref_to_ptr(exc.into());
                    }
                }
            }
        }
    };
}

exc_statics! {
    PyExc_BaseException => "BaseException",
    PyExc_Exception => "Exception",
    PyExc_StopAsyncIteration => "StopAsyncIteration",
    PyExc_StopIteration => "StopIteration",
    PyExc_GeneratorExit => "GeneratorExit",
    PyExc_ArithmeticError => "ArithmeticError",
    PyExc_LookupError => "LookupError",
    PyExc_AssertionError => "AssertionError",
    PyExc_AttributeError => "AttributeError",
    PyExc_BufferError => "BufferError",
    PyExc_EOFError => "EOFError",
    PyExc_FloatingPointError => "FloatingPointError",
    PyExc_OSError => "OSError",
    PyExc_ImportError => "ImportError",
    PyExc_ModuleNotFoundError => "ModuleNotFoundError",
    PyExc_IndexError => "IndexError",
    PyExc_KeyError => "KeyError",
    PyExc_KeyboardInterrupt => "KeyboardInterrupt",
    PyExc_MemoryError => "MemoryError",
    PyExc_NameError => "NameError",
    PyExc_OverflowError => "OverflowError",
    PyExc_RuntimeError => "RuntimeError",
    PyExc_RecursionError => "RecursionError",
    PyExc_NotImplementedError => "NotImplementedError",
    PyExc_SyntaxError => "SyntaxError",
    PyExc_IndentationError => "IndentationError",
    PyExc_TabError => "TabError",
    PyExc_ReferenceError => "ReferenceError",
    PyExc_SystemError => "SystemError",
    PyExc_SystemExit => "SystemExit",
    PyExc_TypeError => "TypeError",
    PyExc_UnboundLocalError => "UnboundLocalError",
    PyExc_UnicodeError => "UnicodeError",
    PyExc_UnicodeEncodeError => "UnicodeEncodeError",
    PyExc_UnicodeDecodeError => "UnicodeDecodeError",
    PyExc_UnicodeTranslateError => "UnicodeTranslateError",
    PyExc_ValueError => "ValueError",
    PyExc_ZeroDivisionError => "ZeroDivisionError",
    PyExc_BlockingIOError => "BlockingIOError",
    PyExc_BrokenPipeError => "BrokenPipeError",
    PyExc_ChildProcessError => "ChildProcessError",
    PyExc_ConnectionError => "ConnectionError",
    PyExc_ConnectionAbortedError => "ConnectionAbortedError",
    PyExc_ConnectionRefusedError => "ConnectionRefusedError",
    PyExc_ConnectionResetError => "ConnectionResetError",
    PyExc_FileExistsError => "FileExistsError",
    PyExc_FileNotFoundError => "FileNotFoundError",
    PyExc_InterruptedError => "InterruptedError",
    PyExc_IsADirectoryError => "IsADirectoryError",
    PyExc_NotADirectoryError => "NotADirectoryError",
    PyExc_PermissionError => "PermissionError",
    PyExc_ProcessLookupError => "ProcessLookupError",
    PyExc_TimeoutError => "TimeoutError",
    PyExc_Warning => "Warning",
    PyExc_UserWarning => "UserWarning",
    PyExc_DeprecationWarning => "DeprecationWarning",
    PyExc_PendingDeprecationWarning => "PendingDeprecationWarning",
    PyExc_SyntaxWarning => "SyntaxWarning",
    PyExc_RuntimeWarning => "RuntimeWarning",
    PyExc_FutureWarning => "FutureWarning",
    PyExc_ImportWarning => "ImportWarning",
    PyExc_UnicodeWarning => "UnicodeWarning",
    PyExc_BytesWarning => "BytesWarning",
    PyExc_ResourceWarning => "ResourceWarning",
    PyExc_EncodingWarning => "EncodingWarning",
    PyExc_BaseExceptionGroup => "BaseExceptionGroup"
}

#[allow(non_upper_case_globals)]
pub static mut PyExc_EnvironmentError: *mut PyObject = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut PyExc_IOError: *mut PyObject = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut PyExc_WindowsError: *mut PyObject = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut PyExc_RecursionErrorInst: *mut PyObject = std::ptr::null_mut();

#[inline]
pub unsafe fn PyErr_NoMemory() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let exc = vm.new_memory_error("out of memory");
        let ptr = pyobject_ref_to_ptr(exc.clone().into());
        set_current_exception(Some(exc));
        ptr
    })
}

#[inline]
pub unsafe fn PyErr_NewExceptionWithDoc(
    name: *const c_char,
    doc: *const c_char,
    base: *mut PyObject,
    dict: *mut PyObject,
) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let full_name = cstr_to_string(name);
        let (module, class_name) = full_name
            .rsplit_once('.')
            .map(|(m, c)| (m, c))
            .unwrap_or(("builtins", full_name.as_str()));
        let bases = if base.is_null() {
            None
        } else {
            Some(vec![
                match ptr_to_pyobject_ref_borrowed(base).downcast::<PyType>() {
                    Ok(base_ty) => base_ty,
                    Err(_) => return std::ptr::null_mut(),
                },
            ])
        };
        let exc = vm.ctx.new_exception_type(module, class_name, bases);
        if !doc.is_null() {
            exc.set_attr(
                vm.ctx.intern_str("__doc__"),
                vm.ctx.new_str(cstr_to_string(doc)).into(),
            );
        }
        let _ = dict;
        pyobject_ref_to_ptr(exc.into())
    })
}

#[inline]
pub unsafe fn PyErr_WriteUnraisable(obj: *mut PyObject) {
    let Some(exc) = take_current_exception() else {
        return;
    };

    rustpython_runtime::with_vm(|vm| {
        let object = if obj.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(obj)
        };
        vm.run_unraisable(exc, None, object);
    });
}

#[inline]
pub unsafe fn PyErr_CheckSignals() -> c_int {
    0
}

fn set_current_exception(exc: Option<PyBaseExceptionRef>) {
    let new_ptr = exc.map(|exc| pyobject_ref_to_ptr(exc.into()) as usize);
    let mut slot = current_exception_slot()
        .lock()
        .expect("RustPython exception slot poisoned");
    let old_ptr = std::mem::replace(&mut *slot, new_ptr);
    drop(slot);
    if let Some(old_ptr) = old_ptr {
        unsafe { crate::Py_DECREF(old_ptr as *mut PyObject) };
    }
}

pub(crate) fn set_vm_exception(exc: PyBaseExceptionRef) {
    set_current_exception(Some(exc));
}

pub(crate) fn set_vm_error(err: PyBaseExceptionRef) {
    set_current_exception(Some(err));
}

pub(crate) fn clear_vm_exception() {
    set_current_exception(None);
}

fn take_current_exception() -> Option<PyBaseExceptionRef> {
    let ptr = current_exception_slot()
        .lock()
        .expect("RustPython exception slot poisoned")
        .take()? as *mut PyObject;
    match unsafe { ptr_to_pyobject_ref_owned(ptr) }.downcast::<PyBaseException>() {
        Ok(exc) => Some(exc),
        Err(obj) => {
            unsafe { crate::Py_DECREF(pyobject_ref_to_ptr(obj)) };
            None
        }
    }
}

fn current_exception() -> Option<PyBaseExceptionRef> {
    let ptr = (*current_exception_slot()
        .lock()
        .expect("RustPython exception slot poisoned"))? as *mut PyObject;
    unsafe { crate::object::Py_IncRef(ptr) };
    match unsafe { ptr_to_pyobject_ref_owned(ptr) }.downcast::<PyBaseException>() {
        Ok(exc) => Some(exc),
        Err(obj) => {
            unsafe { crate::Py_DECREF(pyobject_ref_to_ptr(obj)) };
            None
        }
    }
}

unsafe fn normalize_exception_triplet(
    ptype: *mut PyObject,
    pvalue: *mut PyObject,
    ptraceback: *mut PyObject,
) -> Result<PyBaseExceptionRef, PyBaseExceptionRef> {
    rustpython_runtime::with_vm(|vm| {
        let value = if pvalue.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(pvalue)
        };
        let tb = if ptraceback.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(ptraceback)
        };
        let ty = if ptype.is_null() {
            if let Ok(exc) = value.clone().downcast::<PyBaseException>() {
                exc.class().to_owned().into()
            } else {
                value.clone()
            }
        } else {
            ptr_to_pyobject_ref_borrowed(ptype)
        };
        vm.normalize_exception(ty, value, tb)
    })
}

unsafe fn resolve_exception_type_ptr(exception: *mut PyObject) -> Option<PyObjectRef> {
    if exception.is_null() {
        None
    } else {
        Some(ptr_to_pyobject_ref_borrowed(exception))
    }
}

fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

#[inline]
pub unsafe fn PyErr_SetNone(arg1: *mut PyObject) {
    rustpython_runtime::with_vm(|vm| {
        if let Some(ty) = resolve_exception_type_ptr(arg1) {
            match vm.normalize_exception(ty, vm.ctx.none(), vm.ctx.none()) {
                Ok(exc) => set_current_exception(Some(exc)),
                Err(exc) => set_current_exception(Some(exc)),
            }
        } else {
            set_current_exception(None);
        }
    });
}

#[inline]
pub unsafe fn PyErr_SetObject(arg1: *mut PyObject, arg2: *mut PyObject) {
    match normalize_exception_triplet(arg1, arg2, std::ptr::null_mut()) {
        Ok(exc) | Err(exc) => set_current_exception(Some(exc)),
    }
}

#[inline]
pub unsafe fn PyErr_SetString(exception: *mut PyObject, string: *const c_char) {
    rustpython_runtime::with_vm(|vm| {
        let message = vm.ctx.new_str(cstr_to_string(string));
        match vm.normalize_exception(
            ptr_to_pyobject_ref_borrowed(exception),
            message.into(),
            vm.ctx.none(),
        ) {
            Ok(exc) | Err(exc) => set_current_exception(Some(exc)),
        }
    });
}

#[inline]
pub unsafe fn PyErr_Occurred() -> *mut PyObject {
    current_exception()
        .map(|exc| pyobject_ref_to_ptr(exc.class().to_owned().into()))
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyErr_Clear() {
    set_current_exception(None);
}

#[inline]
pub unsafe fn PyErr_Fetch(
    arg1: *mut *mut PyObject,
    arg2: *mut *mut PyObject,
    arg3: *mut *mut PyObject,
) {
    let current = take_current_exception();
    match current {
        Some(exc) => {
            let (ty, value, tb) = rustpython_runtime::with_vm(|vm| vm.split_exception(exc));
            if !arg1.is_null() {
                *arg1 = pyobject_ref_to_ptr(ty);
            }
            if !arg2.is_null() {
                *arg2 = pyobject_ref_to_ptr(value);
            }
            if !arg3.is_null() {
                *arg3 = pyobject_ref_to_ptr(tb);
            }
        }
        None => {
            if !arg1.is_null() {
                *arg1 = std::ptr::null_mut();
            }
            if !arg2.is_null() {
                *arg2 = std::ptr::null_mut();
            }
            if !arg3.is_null() {
                *arg3 = std::ptr::null_mut();
            }
        }
    }
}

#[inline]
pub unsafe fn PyErr_Restore(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) {
    if arg1.is_null() && arg2.is_null() && arg3.is_null() {
        set_current_exception(None);
        return;
    }
    match normalize_exception_triplet(arg1, arg2, arg3) {
        Ok(exc) | Err(exc) => set_current_exception(Some(exc)),
    }
}

#[inline]
pub unsafe fn PyErr_GetExcInfo(
    arg1: *mut *mut PyObject,
    arg2: *mut *mut PyObject,
    arg3: *mut *mut PyObject,
) {
    PyErr_Fetch(arg1, arg2, arg3);
}

#[inline]
pub unsafe fn PyErr_SetExcInfo(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) {
    PyErr_Restore(arg1, arg2, arg3)
}

#[inline]
pub unsafe fn Py_FatalError(message: *const c_char) -> ! {
    panic!("{}", cstr_to_string(message))
}

#[inline]
pub unsafe fn PyErr_GivenExceptionMatches(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int {
    if arg1.is_null() || arg2.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let given = ptr_to_pyobject_ref_borrowed(arg1);
        let expected = ptr_to_pyobject_ref_borrowed(arg2);

        if let Ok(tuple) = expected.clone().downcast::<PyTuple>() {
            return tuple.as_slice().iter().any(|candidate| unsafe {
                PyErr_GivenExceptionMatches(arg1, pyobject_ref_as_ptr(candidate)) != 0
            }) as c_int;
        }

        let given_type = if let Ok(exc) = given.clone().downcast::<PyBaseException>() {
            exc.class().to_owned().into()
        } else {
            given
        };
        let Ok(expected_ctor) = ExceptionCtor::try_from_object(vm, expected) else {
            return 0;
        };
        match expected_ctor {
            ExceptionCtor::Class(cls) => given_type
                .real_is_subclass(cls.as_object(), vm)
                .unwrap_or(false) as c_int,
            ExceptionCtor::Instance(exc) => given_type
                .real_is_subclass(exc.class().as_object(), vm)
                .unwrap_or(false) as c_int,
        }
    })
}

#[inline]
pub unsafe fn PyErr_ExceptionMatches(arg1: *mut PyObject) -> c_int {
    current_exception()
        .map(|exc| PyErr_GivenExceptionMatches(pyobject_ref_to_ptr(exc.into()), arg1))
        .unwrap_or(0)
}

#[inline]
pub unsafe fn PyErr_NormalizeException(
    arg1: *mut *mut PyObject,
    arg2: *mut *mut PyObject,
    arg3: *mut *mut PyObject,
) {
    if arg1.is_null() || arg2.is_null() || arg3.is_null() {
        return;
    }
    match normalize_exception_triplet(*arg1, *arg2, *arg3) {
        Ok(exc) | Err(exc) => {
            rustpython_runtime::with_vm(|vm| {
                let (ty, value, tb) = vm.split_exception(exc);
                *arg1 = pyobject_ref_to_ptr(ty);
                *arg2 = pyobject_ref_to_ptr(value);
                *arg3 = pyobject_ref_to_ptr(tb);
            });
        }
    }
}

#[inline]
pub unsafe fn PyErr_GetRaisedException() -> *mut PyObject {
    let current = take_current_exception();
    current
        .map(|exc| pyobject_ref_to_ptr(exc.into()))
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyErr_SetRaisedException(exc: *mut PyObject) {
    if exc.is_null() {
        set_current_exception(None);
        return;
    }
    // CPython steals a reference here, so adopt the incoming pointer rather than
    // cloning it as a borrowed reference.
    let obj = ptr_to_pyobject_ref_owned(exc);
    rustpython_runtime::with_vm(|vm| match ExceptionCtor::try_from_object(vm, obj) {
        Ok(ctor) => match ctor.instantiate(vm) {
            Ok(exc) => set_current_exception(Some(exc)),
            Err(exc) => set_current_exception(Some(exc)),
        },
        Err(exc) => set_current_exception(Some(exc)),
    });
}

#[cfg(Py_3_11)]
#[inline]
pub unsafe fn PyErr_GetHandledException() -> *mut PyObject {
    PyErr_GetRaisedException()
}

#[cfg(Py_3_11)]
#[inline]
pub unsafe fn PyErr_SetHandledException(exc: *mut PyObject) {
    PyErr_SetRaisedException(exc)
}

#[inline]
pub unsafe fn PyException_SetTraceback(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int {
    if arg1.is_null() {
        return -1;
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        let value = if arg2.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(arg2)
        };
        exc.set_attr("__traceback__", value, vm)
            .map(|_| 0)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PyException_GetTraceback(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        exc.get_attr("__traceback__", vm)
            .ok()
            .filter(|value| !value.is(&vm.ctx.none()))
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyException_GetCause(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        exc.get_attr("__cause__", vm)
            .ok()
            .filter(|value| !value.is(&vm.ctx.none()))
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyException_SetCause(arg1: *mut PyObject, arg2: *mut PyObject) {
    if arg1.is_null() {
        return;
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        let cause = if arg2.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(arg2)
        };
        let _ = exc.set_attr("__cause__", cause, vm);
    });
}

#[inline]
pub unsafe fn PyException_GetContext(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        exc.get_attr("__context__", vm)
            .ok()
            .filter(|value| !value.is(&vm.ctx.none()))
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyException_SetContext(arg1: *mut PyObject, arg2: *mut PyObject) {
    if arg1.is_null() {
        return;
    }
    rustpython_runtime::with_vm(|vm| {
        let exc = ptr_to_pyobject_ref_borrowed(arg1);
        let ctx = if arg2.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(arg2)
        };
        let _ = exc.set_attr("__context__", ctx, vm);
    });
}

#[inline]
pub unsafe fn PyExceptionInstance_Class(x: *mut PyObject) -> *mut PyObject {
    if x.is_null() {
        return std::ptr::null_mut();
    }
    pyobject_ref_to_ptr(ptr_to_pyobject_ref_borrowed(x).class().to_owned().into())
}

#[inline]
pub unsafe fn PyExceptionClass_Check(x: *mut PyObject) -> c_int {
    if x.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(x);
        obj.downcast::<PyType>()
            .map(|cls| cls.fast_issubclass(vm.ctx.exceptions.base_exception_type) as c_int)
            .unwrap_or(0)
    })
}

#[inline]
pub unsafe fn PyExceptionInstance_Check(x: *mut PyObject) -> c_int {
    if x.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        ptr_to_pyobject_ref_borrowed(x)
            .class()
            .fast_issubclass(vm.ctx.exceptions.base_exception_type) as c_int
    })
}

#[inline]
pub unsafe fn PyUnicodeDecodeError_Create(
    encoding: *const c_char,
    object: *const c_char,
    length: Py_ssize_t,
    start: Py_ssize_t,
    end: Py_ssize_t,
    reason: *const c_char,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let exc = vm.new_unicode_decode_error_real(
            vm.ctx.new_str(cstr_to_string(encoding)),
            vm.ctx.new_bytes(if object.is_null() || length < 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(object.cast::<u8>(), length as usize).to_vec()
            }),
            start.max(0) as usize,
            end.max(0) as usize,
            vm.ctx.new_str(cstr_to_string(reason)),
        );
        pyobject_ref_to_ptr(exc.into())
    })
}
