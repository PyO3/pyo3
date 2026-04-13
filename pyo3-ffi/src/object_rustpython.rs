use crate::pyport::{Py_hash_t, Py_ssize_t};
use crate::rustpython_runtime;
use crate::{methodobject, pyerrors::{PyErr_GetRaisedException, set_vm_exception}};
use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

use rustpython_vm::builtins::{
    PyBaseException, PyBaseObject, PyDict, PyList, PySet, PyStr, PyType, PyTypeRef,
};
use rustpython_vm::function::{FuncArgs, PyMethodDef as RpMethodDef, PyMethodFlags as RpMethodFlags};
use rustpython_vm::protocol::{PyMapping, PySequence};
use rustpython_vm::types::PyComparisonOp;
use rustpython_vm::types::{Constructor, PyTypeFlags, PyTypeSlots};
use rustpython_vm::{AsObject, PyObjectRef, PyPayload};

#[repr(C)]
#[derive(Debug)]
pub struct PyObject {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
}

#[repr(C)]
#[derive(Debug)]
pub struct PyTypeObject {
    pub(crate) _opaque: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct PyVarObject {
    pub ob_base: PyObject,
    pub ob_size: Py_ssize_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyType_Slot {
    pub slot: c_int,
    pub pfunc: *mut c_void,
}

impl Default for PyType_Slot {
    fn default() -> Self {
        Self {
            slot: 0,
            pfunc: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyType_Spec {
    pub name: *const c_char,
    pub basicsize: c_int,
    pub itemsize: c_int,
    pub flags: c_uint,
    pub slots: *mut PyType_Slot,
}

impl Default for PyType_Spec {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            basicsize: 0,
            itemsize: 0,
            flags: 0,
            slots: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct _PyWeakReference {
    _opaque: [u8; 0],
}

pub type PyTupleObject = PyObject;

pub type unaryfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type binaryfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
pub type ternaryfunc = unsafe extern "C" fn(
    arg1: *mut PyObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject;
pub type inquiry = unsafe extern "C" fn(arg1: *mut PyObject) -> c_int;
pub type lenfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> Py_ssize_t;
pub type ssizeargfunc = unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
pub type ssizeobjargproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int;
pub type objobjproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
pub type objobjargproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type destructor = unsafe extern "C" fn(arg1: *mut PyObject);
pub type getattrfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *const c_char) -> *mut PyObject;
pub type setattrfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *const c_char, arg3: *mut PyObject) -> c_int;
pub type reprfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type hashfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> Py_hash_t;
pub type getattrofunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
pub type setattrofunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type traverseproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: visitproc, arg3: *mut c_void) -> c_int;
pub type richcmpfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: c_int) -> *mut PyObject;
pub type getiterfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type iternextfunc = unsafe extern "C" fn(arg1: *mut PyObject) -> *mut PyObject;
pub type descrgetfunc = unsafe extern "C" fn(
    arg1: *mut PyObject,
    arg2: *mut PyObject,
    arg3: *mut PyObject,
) -> *mut PyObject;
pub type descrsetfunc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type initproc =
    unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> c_int;
pub type allocfunc =
    unsafe extern "C" fn(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyObject;
pub type newfunc =
    unsafe extern "C" fn(arg1: *mut PyTypeObject, arg2: *mut PyObject, arg3: *mut PyObject)
        -> *mut PyObject;
pub type freefunc = unsafe extern "C" fn(arg1: *mut c_void);
pub type visitproc = unsafe extern "C" fn(arg1: *mut PyObject, arg2: *mut c_void) -> c_int;
pub type vectorcallfunc = unsafe extern "C" fn(
    callable: *mut PyObject,
    args: *const *mut PyObject,
    nargsf: usize,
    kwnames: *mut PyObject,
) -> *mut PyObject;

#[repr(C)]
#[derive(Clone, Default)]
pub struct PyBufferProcs {
    pub bf_getbuffer: Option<crate::getbufferproc>,
    pub bf_releasebuffer: Option<crate::releasebufferproc>,
}

#[allow(non_upper_case_globals)]
pub static mut PyType_Type: PyTypeObject = PyTypeObject { _opaque: [] };
#[allow(non_upper_case_globals)]
pub static mut PyBaseObject_Type: PyTypeObject = PyTypeObject { _opaque: [] };
#[allow(non_upper_case_globals)]
pub static mut PyLong_Type: PyTypeObject = PyTypeObject { _opaque: [] };
#[allow(non_upper_case_globals)]
pub static mut PyBool_Type: PyTypeObject = PyTypeObject { _opaque: [] };

pub const PyObject_HEAD_INIT: PyObject = PyObject {
    ob_refcnt: 0,
    ob_type: std::ptr::null_mut(),
};

#[derive(Copy, Clone, Default)]
struct HeapTypeMetadata {
    tp_new: usize,
    tp_init: usize,
    tp_call: usize,
    mp_subscript: usize,
    mp_ass_subscript: usize,
    nb_add: usize,
    sq_length: usize,
    sq_item: usize,
    sq_ass_item: usize,
    sq_contains: usize,
    sq_concat: usize,
    sq_repeat: usize,
    sq_inplace_concat: usize,
    sq_inplace_repeat: usize,
}

fn heap_type_registry() -> &'static Mutex<std::collections::HashMap<usize, HeapTypeMetadata>> {
    static REGISTRY: OnceLock<Mutex<std::collections::HashMap<usize, HeapTypeMetadata>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn ffi_name_to_static(ptr: *const c_char, default: &'static str) -> &'static str {
    if ptr.is_null() {
        return default;
    }
    let owned = unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
        .into_boxed_str();
    Box::leak(owned)
}

const SIGNATURE_END_MARKER: &str = ")\n--\n\n";

fn find_internal_doc_signature<'a>(name: &str, doc: &'a str) -> Option<&'a str> {
    let name = name.rsplit('.').next().unwrap_or(name);
    let doc = doc.strip_prefix(name)?;
    doc.starts_with('(').then_some(doc)
}

fn doc_from_internal_doc<'a>(name: &str, internal_doc: &'a str) -> &'a str {
    if let Some(doc_without_sig) = find_internal_doc_signature(name, internal_doc) {
        if let Some(sig_end_pos) = doc_without_sig.find(SIGNATURE_END_MARKER) {
            return &doc_without_sig[sig_end_pos + SIGNATURE_END_MARKER.len()..];
        }
    }
    internal_doc
}

unsafe fn fetch_current_exception(
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::builtins::PyBaseExceptionRef {
    let raised = PyErr_GetRaisedException();
    if raised.is_null() {
        return vm.new_system_error("FFI callback returned NULL without setting an exception");
    }
    match ptr_to_pyobject_ref_owned(raised).downcast::<PyBaseException>() {
        Ok(exc) => exc,
        Err(obj) => vm.new_system_error(format!(
            "FFI callback set a non-exception object: {}",
            obj.class().name()
        )),
    }
}

fn build_func_args_from_ffi(
    args: *mut PyObject,
    kwargs: *mut PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<FuncArgs> {
    let positional = if args.is_null() {
        Vec::new()
    } else {
        let args_obj = unsafe { ptr_to_pyobject_ref_borrowed(args) };
        args_obj
            .try_into_value::<rustpython_vm::builtins::PyTupleRef>(vm)
            .map(|tuple| tuple.as_slice().to_vec())
            .map_err(|_| vm.new_type_error("expected tuple args for tp_new"))?
    };

    let mut kw = rustpython_vm::function::KwArgs::default();
    if !kwargs.is_null() {
        let kwargs_obj = unsafe { ptr_to_pyobject_ref_borrowed(kwargs) };
        let kwargs_dict = kwargs_obj
            .try_into_value::<rustpython_vm::builtins::PyDictRef>(vm)
            .map_err(|_| vm.new_type_error("expected dict kwargs for tp_new"))?;
        for (k, v) in &kwargs_dict {
            let key = k
                .str(vm)
                .map_err(|_| vm.new_type_error("keywords must be strings"))?;
            kw = std::iter::once((AsRef::<str>::as_ref(&key).to_owned(), v))
                .chain(kw)
                .collect();
        }
    }

    Ok(FuncArgs::new(positional, kw))
}

fn build_getter_property(
    def: *mut crate::descrobject::PyGetSetDef,
    vm: &rustpython_vm::VirtualMachine,
) -> PyObjectRef {
    let name = ffi_name_to_static(unsafe { (*def).name }, "<getter>");
    let getter = unsafe { (*def).get };
    let setter = unsafe { (*def).set };
    let closure = unsafe { (*def).closure as usize };
    let doc = unsafe { (*def).doc };

    let fget = getter.map(|get| {
        let def_name = name;
        let method = Box::leak(Box::new(RpMethodDef {
            name: def_name,
            func: Box::leak(Box::new(move |vm: &rustpython_vm::VirtualMachine, args: FuncArgs| {
                if args.args.len() != 1 || !args.kwargs.is_empty() {
                    return Err(vm.new_type_error(format!(
                        "{def_name} getter expects exactly one argument"
                    )));
                }
                let obj = &args.args[0];
                let result = unsafe {
                    get(pyobject_ref_as_ptr(obj), closure as *mut c_void)
                };
                if result.is_null() {
                    Err(unsafe { fetch_current_exception(vm) })
                } else {
                    Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
                }
            })),
            flags: RpMethodFlags::EMPTY,
            doc: None,
        }));
        method.build_function(&vm.ctx).into()
    });

    let fset = setter.map(|set| {
        let def_name = name;
        let method = Box::leak(Box::new(RpMethodDef {
            name: def_name,
            func: Box::leak(Box::new(move |vm: &rustpython_vm::VirtualMachine, args: FuncArgs| {
                if args.args.len() != 2 || !args.kwargs.is_empty() {
                    return Err(vm.new_type_error(format!(
                        "{def_name} setter expects exactly two arguments"
                    )));
                }
                let obj = &args.args[0];
                let value = &args.args[1];
                let rc = unsafe {
                    set(
                        pyobject_ref_as_ptr(obj),
                        pyobject_ref_as_ptr(value),
                        closure as *mut c_void,
                    )
                };
                if rc == 0 {
                    Ok(vm.ctx.none())
                } else {
                    Err(unsafe { fetch_current_exception(vm) })
                }
            })),
            flags: RpMethodFlags::EMPTY,
            doc: None,
        }));
        method.build_function(&vm.ctx).into()
    });

    let fdel = setter.map(|set| {
        let def_name = name;
        let method = Box::leak(Box::new(RpMethodDef {
            name: def_name,
            func: Box::leak(Box::new(move |vm: &rustpython_vm::VirtualMachine, args: FuncArgs| {
                if args.args.len() != 1 || !args.kwargs.is_empty() {
                    return Err(vm.new_type_error(format!(
                        "{def_name} deleter expects exactly one argument"
                    )));
                }
                let obj = &args.args[0];
                let rc = unsafe {
                    set(
                        pyobject_ref_as_ptr(obj),
                        std::ptr::null_mut(),
                        closure as *mut c_void,
                    )
                };
                if rc == 0 {
                    Ok(vm.ctx.none())
                } else {
                    Err(unsafe { fetch_current_exception(vm) })
                }
            })),
            flags: RpMethodFlags::EMPTY,
            doc: None,
        }));
        method.build_function(&vm.ctx).into()
    });

    let doc_obj = if doc.is_null() {
        vm.ctx.none()
    } else {
        vm.ctx.new_str(ffi_name_to_static(doc, "")).into()
    };

    vm.ctx
        .types
        .property_type
        .as_object()
        .call(
            (
                fget.unwrap_or_else(|| vm.ctx.none()),
                fset.unwrap_or_else(|| vm.ctx.none()),
                fdel.unwrap_or_else(|| vm.ctx.none()),
                doc_obj,
            ),
            vm,
        )
        .unwrap()
}

fn heap_tp_new_wrapper(cls: PyTypeRef, args: FuncArgs, vm: &rustpython_vm::VirtualMachine) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = cls.to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(tp_new) = (metadata.tp_new != 0).then_some(metadata.tp_new) else {
        return Err(vm.new_type_error("heap type missing tp_new"));
    };
    let tp_new: newfunc = unsafe { std::mem::transmute(tp_new) };

    let tuple = vm.ctx.new_tuple(args.args);
    let tuple_obj: PyObjectRef = tuple.into();
    let kwargs_obj = if args.kwargs.is_empty() {
        None
    } else {
        let dict = vm.ctx.new_dict();
        for (key, value) in args.kwargs {
            dict.set_item(key.as_str(), value, vm)?;
        }
        Some::<PyObjectRef>(dict.into())
    };

    let result = unsafe {
        tp_new(
            cls_ptr,
            pyobject_ref_as_ptr(&tuple_obj),
            kwargs_obj
                .as_ref()
                .map(pyobject_ref_as_ptr)
                .unwrap_or(std::ptr::null_mut()),
        )
    };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_tp_init_wrapper(
    zelf: PyObjectRef,
    args: FuncArgs,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<()> {
    let cls = zelf.class();
    let cls_obj: PyObjectRef = cls.to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(tp_init) = (metadata.tp_init != 0).then_some(metadata.tp_init) else {
        return Ok(());
    };
    let tp_init: initproc = unsafe { std::mem::transmute(tp_init) };

    let tuple = vm.ctx.new_tuple(args.args);
    let tuple_obj: PyObjectRef = tuple.into();
    let kwargs_obj = if args.kwargs.is_empty() {
        None
    } else {
        let dict = vm.ctx.new_dict();
        for (key, value) in args.kwargs {
            dict.set_item(key.as_str(), value, vm)?;
        }
        Some::<PyObjectRef>(dict.into())
    };

    let rc = unsafe {
        tp_init(
            pyobject_ref_as_ptr(&zelf),
            pyobject_ref_as_ptr(&tuple_obj),
            kwargs_obj
                .as_ref()
                .map(pyobject_ref_as_ptr)
                .unwrap_or(std::ptr::null_mut()),
        )
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(unsafe { fetch_current_exception(vm) })
    }
}

fn heap_mapping_getitem_wrapper(
    mapping: PyMapping<'_>,
    key: &rustpython_vm::PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = mapping.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(mp_subscript) = (metadata.mp_subscript != 0).then_some(metadata.mp_subscript) else {
        return Err(vm.new_type_error(format!(
            "'{}' does not support item access",
            mapping.obj.class()
        )));
    };
    let mp_subscript: binaryfunc = unsafe { std::mem::transmute(mp_subscript) };
    let result = unsafe { mp_subscript(pyobject_ref_as_ptr(&mapping.obj.to_owned()), pyobject_ref_as_ptr(&key.to_owned())) };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_mapping_setitem_wrapper(
    mapping: PyMapping<'_>,
    key: &rustpython_vm::PyObject,
    value: Option<PyObjectRef>,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<()> {
    let cls_obj: PyObjectRef = mapping.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(mp_ass_subscript) =
        (metadata.mp_ass_subscript != 0).then_some(metadata.mp_ass_subscript)
    else {
        return Err(vm.new_type_error(format!(
            "'{}' does not support item assignment",
            mapping.obj.class()
        )));
    };
    let mp_ass_subscript: objobjargproc = unsafe { std::mem::transmute(mp_ass_subscript) };
    let key_obj = key.to_owned();
    let value_ptr = value
        .as_ref()
        .map(pyobject_ref_as_ptr)
        .unwrap_or(std::ptr::null_mut());
    let rc = unsafe {
        mp_ass_subscript(
            pyobject_ref_as_ptr(&mapping.obj.to_owned()),
            pyobject_ref_as_ptr(&key_obj),
            value_ptr,
        )
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(unsafe { fetch_current_exception(vm) })
    }
}

fn heap_nb_add_wrapper(
    lhs: &rustpython_vm::PyObject,
    rhs: &rustpython_vm::PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = lhs.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(nb_add) = (metadata.nb_add != 0).then_some(metadata.nb_add) else {
        return Err(vm.new_type_error(format!(
            "unsupported operand type(s) for +: '{}' and '{}'",
            lhs.class(),
            rhs.class()
        )));
    };
    let nb_add: binaryfunc = unsafe { std::mem::transmute(nb_add) };
    let lhs_obj = lhs.to_owned();
    let rhs_obj = rhs.to_owned();
    let result = unsafe { nb_add(pyobject_ref_as_ptr(&lhs_obj), pyobject_ref_as_ptr(&rhs_obj)) };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_sq_length_wrapper(
    seq: PySequence<'_>,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<usize> {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_length) = (metadata.sq_length != 0).then_some(metadata.sq_length) else {
        return Err(vm.new_type_error(format!(
            "object of type '{}' has no len()",
            seq.obj.class()
        )));
    };
    let sq_length: lenfunc = unsafe { std::mem::transmute(sq_length) };
    let rc = unsafe { sq_length(pyobject_ref_as_ptr(&seq.obj.to_owned())) };
    if rc < 0 {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(rc as usize)
    }
}

fn heap_sq_item_wrapper(
    seq: PySequence<'_>,
    index: isize,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_item) = (metadata.sq_item != 0).then_some(metadata.sq_item) else {
        return Err(vm.new_type_error(format!(
            "'{}' is not a sequence or does not support indexing",
            seq.obj.class()
        )));
    };
    let sq_item: ssizeargfunc = unsafe { std::mem::transmute(sq_item) };
    let result =
        unsafe { sq_item(pyobject_ref_as_ptr(&seq.obj.to_owned()), index as Py_ssize_t) };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_sq_ass_item_wrapper(
    seq: PySequence<'_>,
    index: isize,
    value: Option<PyObjectRef>,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<()> {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_ass_item) = (metadata.sq_ass_item != 0).then_some(metadata.sq_ass_item) else {
        return Err(vm.new_type_error(format!(
            "'{}' is not a sequence or doesn't support item assignment",
            seq.obj.class()
        )));
    };
    let sq_ass_item: ssizeobjargproc = unsafe { std::mem::transmute(sq_ass_item) };
    let value_ptr = value
        .as_ref()
        .map(pyobject_ref_as_ptr)
        .unwrap_or(std::ptr::null_mut());
    let rc = unsafe {
        sq_ass_item(
            pyobject_ref_as_ptr(&seq.obj.to_owned()),
            index as Py_ssize_t,
            value_ptr,
        )
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(unsafe { fetch_current_exception(vm) })
    }
}

fn heap_sq_contains_wrapper(
    seq: PySequence<'_>,
    needle: &rustpython_vm::PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<bool> {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_contains) = (metadata.sq_contains != 0).then_some(metadata.sq_contains) else {
        return Err(vm.new_type_error(format!(
            "argument of type '{}' is not iterable",
            seq.obj.class()
        )));
    };
    let sq_contains: objobjproc = unsafe { std::mem::transmute(sq_contains) };
    let needle_obj = needle.to_owned();
    let rc = unsafe {
        sq_contains(
            pyobject_ref_as_ptr(&seq.obj.to_owned()),
            pyobject_ref_as_ptr(&needle_obj),
        )
    };
    if rc < 0 {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(rc != 0)
    }
}

fn heap_sq_concat_wrapper(
    seq: PySequence<'_>,
    other: &rustpython_vm::PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_concat) = (metadata.sq_concat != 0).then_some(metadata.sq_concat) else {
        return Err(vm.new_type_error(format!(
            "'{}' object can't be concatenated",
            seq.obj.class()
        )));
    };
    let sq_concat: binaryfunc = unsafe { std::mem::transmute(sq_concat) };
    let seq_obj = seq.obj.to_owned();
    let other_obj = other.to_owned();
    let result =
        unsafe { sq_concat(pyobject_ref_as_ptr(&seq_obj), pyobject_ref_as_ptr(&other_obj)) };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_sq_repeat_wrapper(
    seq: PySequence<'_>,
    count: isize,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_repeat) = (metadata.sq_repeat != 0).then_some(metadata.sq_repeat) else {
        return Err(vm.new_type_error(format!(
            "'{}' object can't be repeated",
            seq.obj.class()
        )));
    };
    let sq_repeat: ssizeargfunc = unsafe { std::mem::transmute(sq_repeat) };
    let result =
        unsafe { sq_repeat(pyobject_ref_as_ptr(&seq.obj.to_owned()), count as Py_ssize_t) };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_sq_inplace_concat_wrapper(
    seq: PySequence<'_>,
    other: &rustpython_vm::PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_inplace_concat) = (metadata.sq_inplace_concat != 0)
        .then_some(metadata.sq_inplace_concat)
    else {
        return heap_sq_concat_wrapper(seq, other, vm);
    };
    let sq_inplace_concat: binaryfunc = unsafe { std::mem::transmute(sq_inplace_concat) };
    let seq_obj = seq.obj.to_owned();
    let other_obj = other.to_owned();
    let result = unsafe {
        sq_inplace_concat(pyobject_ref_as_ptr(&seq_obj), pyobject_ref_as_ptr(&other_obj))
    };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_sq_inplace_repeat_wrapper(
    seq: PySequence<'_>,
    count: isize,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = seq.obj.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(sq_inplace_repeat) = (metadata.sq_inplace_repeat != 0)
        .then_some(metadata.sq_inplace_repeat)
    else {
        return heap_sq_repeat_wrapper(seq, count, vm);
    };
    let sq_inplace_repeat: ssizeargfunc = unsafe { std::mem::transmute(sq_inplace_repeat) };
    let result = unsafe {
        sq_inplace_repeat(pyobject_ref_as_ptr(&seq.obj.to_owned()), count as Py_ssize_t)
    };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

fn heap_tp_call_wrapper(
    callable: &rustpython_vm::PyObject,
    args: FuncArgs,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult {
    let cls_obj: PyObjectRef = callable.class().to_owned().into();
    let cls_ptr = pyobject_ref_as_ptr(&cls_obj) as *mut PyTypeObject;
    let metadata = heap_type_registry()
        .lock()
        .unwrap()
        .get(&(cls_ptr as usize))
        .copied()
        .unwrap_or_default();
    let Some(tp_call) = (metadata.tp_call != 0).then_some(metadata.tp_call) else {
        return Err(vm.new_type_error(format!("'{}' object is not callable", callable.class())));
    };
    let tp_call: ternaryfunc = unsafe { std::mem::transmute(tp_call) };

    let callable_obj = callable.to_owned();
    let tuple = vm.ctx.new_tuple(args.args);
    let tuple_obj: PyObjectRef = tuple.into();
    let kwargs_obj = if args.kwargs.is_empty() {
        None
    } else {
        let dict = vm.ctx.new_dict();
        for (key, value) in args.kwargs {
            dict.set_item(key.as_str(), value, vm)?;
        }
        Some::<PyObjectRef>(dict.into())
    };

    let result = unsafe {
        tp_call(
            pyobject_ref_as_ptr(&callable_obj),
            pyobject_ref_as_ptr(&tuple_obj),
            kwargs_obj
                .as_ref()
                .map(pyobject_ref_as_ptr)
                .unwrap_or(std::ptr::null_mut()),
        )
    };
    if result.is_null() {
        Err(unsafe { fetch_current_exception(vm) })
    } else {
        Ok(unsafe { ptr_to_pyobject_ref_owned(result) })
    }
}

unsafe extern "C" fn builtin_set_tp_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    if subtype.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let cls = unsafe { ptr_to_pyobject_ref_borrowed(subtype.cast()) };
        let Ok(cls) = cls.downcast::<PyType>() else {
            return std::ptr::null_mut();
        };
        let Ok(args) = build_func_args_from_ffi(args, kwds, vm) else {
            return std::ptr::null_mut();
        };
        match <PySet as rustpython_vm::types::Constructor>::slot_new(cls, args, vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                crate::pyerrors::set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn builtin_dict_tp_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    if subtype.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let cls = unsafe { ptr_to_pyobject_ref_borrowed(subtype.cast()) };
        let Ok(cls) = cls.downcast::<PyType>() else {
            return std::ptr::null_mut();
        };
        let Ok(args) = build_func_args_from_ffi(args, kwds, vm) else {
            return std::ptr::null_mut();
        };
        match <PyDict as rustpython_vm::types::Constructor>::slot_new(cls, args, vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                crate::pyerrors::set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn builtin_object_tp_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    if subtype.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let cls = unsafe { ptr_to_pyobject_ref_borrowed(subtype.cast()) };
        let Ok(cls) = cls.downcast::<PyType>() else {
            return std::ptr::null_mut();
        };
        let Ok(args) = build_func_args_from_ffi(args, kwds, vm) else {
            return std::ptr::null_mut();
        };
        match <PyBaseObject as rustpython_vm::types::Constructor>::slot_new(cls, args, vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                crate::pyerrors::set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn builtin_exception_tp_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let cls_obj = unsafe { ptr_to_pyobject_ref_borrowed(subtype as *mut PyObject) };
        let Ok(cls) = cls_obj.clone().downcast::<PyType>() else {
            return std::ptr::null_mut();
        };
        let Ok(args) = build_func_args_from_ffi(args, kwds, vm) else {
            return std::ptr::null_mut();
        };
        match <PyBaseException as Constructor>::slot_new(cls, args, vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                crate::pyerrors::set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub fn pyobject_ref_to_ptr(obj: PyObjectRef) -> *mut PyObject {
    obj.into_raw().as_ptr() as *mut PyObject
}

#[inline]
pub fn pyobject_ref_as_ptr(obj: &PyObjectRef) -> *mut PyObject {
    let ptr: *const rustpython_vm::PyObject = &**obj;
    ptr.cast_mut() as *mut PyObject
}

#[inline]
pub unsafe fn ptr_to_pyobject_ref_owned(ptr: *mut PyObject) -> PyObjectRef {
    let nn = NonNull::new_unchecked(ptr as *mut rustpython_vm::PyObject);
    PyObjectRef::from_raw(nn)
}

#[inline]
pub unsafe fn ptr_to_pyobject_ref_borrowed(ptr: *mut PyObject) -> PyObjectRef {
    let obj = ptr_to_pyobject_ref_owned(ptr);
    let cloned = obj.clone();
    std::mem::forget(obj);
    cloned
}

#[inline]
pub unsafe fn Py_Is(x: *mut PyObject, y: *mut PyObject) -> c_int {
    (x == y).into()
}

#[inline]
pub unsafe fn Py_TYPE(ob: *mut PyObject) -> *mut PyTypeObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    let objref = ptr_to_pyobject_ref_borrowed(ob);
    let typeref: PyObjectRef = objref.class().to_owned().into();
    pyobject_ref_to_ptr(typeref) as *mut PyTypeObject
}

#[inline]
pub unsafe fn Py_SIZE(_ob: *mut PyObject) -> Py_ssize_t {
    0
}

#[inline]
pub unsafe fn Py_IS_TYPE(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_TYPE(ob) == tp) as c_int
}

#[inline]
pub unsafe fn Py_DECREF(obj: *mut PyObject) {
    if obj.is_null() {
        return;
    }
    let _ = ptr_to_pyobject_ref_owned(obj);
}

#[inline]
pub unsafe fn Py_IncRef(obj: *mut PyObject) {
    if obj.is_null() {
        return;
    }
    let obj = ptr_to_pyobject_ref_borrowed(obj);
    std::mem::forget(obj);
}

#[inline]
pub unsafe fn PyTuple_SET_ITEM(_obj: *mut PyObject, _index: Py_ssize_t, _value: *mut PyObject) {}

#[inline]
pub unsafe fn PyTuple_GET_ITEM(_obj: *mut PyObject, _index: Py_ssize_t) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyTuple_GET_SIZE(obj: *mut PyObject) -> Py_ssize_t {
    if obj.is_null() {
        return 0;
    }
    let objref = ptr_to_pyobject_ref_borrowed(obj);
    rustpython_runtime::with_vm(|vm| match objref.length(vm) {
        Ok(len) => len as Py_ssize_t,
        Err(_) => 0,
    })
}

#[inline]
pub unsafe fn PyType_IsSubtype(
    subtype: *mut PyTypeObject,
    supertype: *mut PyTypeObject,
) -> c_int {
    if subtype.is_null() || supertype.is_null() {
        return 0;
    }
    let sub = ptr_to_pyobject_ref_borrowed(subtype as *mut PyObject);
    let sup = ptr_to_pyobject_ref_borrowed(supertype as *mut PyObject);
    rustpython_runtime::with_vm(|vm| match sub.real_is_subclass(&sup, vm) {
        Ok(true) => 1,
        _ => 0,
    })
}

#[inline]
pub unsafe fn PyObject_TypeCheck(ob: *mut PyObject, tp: *mut PyTypeObject) -> c_int {
    (Py_IS_TYPE(ob, tp) != 0 || PyType_IsSubtype(Py_TYPE(ob), tp) != 0) as c_int
}

#[inline]
pub unsafe fn PyObject_IsInstance(ob: *mut PyObject, tp: *mut PyObject) -> c_int {
    if ob.is_null() || tp.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    let typ = ptr_to_pyobject_ref_borrowed(tp);
    rustpython_runtime::with_vm(|vm| {
        if let Ok(typ_type) = typ.try_to_ref::<PyType>(vm) {
            return if obj.class().fast_issubclass(typ_type.as_object()) {
                1
            } else {
                0
            };
        }
        match obj.is_instance(&typ, vm) {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(_) => -1,
        }
    })
}

#[inline]
pub unsafe fn PyObject_IsSubclass(derived: *mut PyObject, cls: *mut PyObject) -> c_int {
    if derived.is_null() || cls.is_null() {
        return -1;
    }
    let derived = ptr_to_pyobject_ref_borrowed(derived);
    let cls = ptr_to_pyobject_ref_borrowed(cls);
    rustpython_runtime::with_vm(|vm| match derived.real_is_subclass(&cls, vm) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    })
}

#[inline]
pub unsafe fn PyObject_Str(ob: *mut PyObject) -> *mut PyObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.str(vm) {
        Ok(s) => pyobject_ref_to_ptr(s.into()),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
fn compare_op_from_raw(op: c_int) -> Option<PyComparisonOp> {
    match op {
        Py_LT => Some(PyComparisonOp::Lt),
        Py_LE => Some(PyComparisonOp::Le),
        Py_EQ => Some(PyComparisonOp::Eq),
        Py_NE => Some(PyComparisonOp::Ne),
        Py_GT => Some(PyComparisonOp::Gt),
        Py_GE => Some(PyComparisonOp::Ge),
        _ => None,
    }
}

#[inline]
pub unsafe fn PyObject_Repr(ob: *mut PyObject) -> *mut PyObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.repr(vm) {
        Ok(s) => pyobject_ref_to_ptr(s.into()),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyObject_RichCompare(
    left: *mut PyObject,
    right: *mut PyObject,
    op: c_int,
) -> *mut PyObject {
    if left.is_null() || right.is_null() {
        return std::ptr::null_mut();
    }
    let Some(op) = compare_op_from_raw(op) else {
        return std::ptr::null_mut();
    };
    let lhs = ptr_to_pyobject_ref_borrowed(left);
    let rhs = ptr_to_pyobject_ref_borrowed(right);
    rustpython_runtime::with_vm(|vm| match lhs.rich_compare(rhs, op, vm) {
        Ok(obj) => pyobject_ref_to_ptr(obj),
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyObject_RichCompareBool(
    left: *mut PyObject,
    right: *mut PyObject,
    op: c_int,
) -> c_int {
    if left.is_null() || right.is_null() {
        return -1;
    }
    let Some(op) = compare_op_from_raw(op) else {
        return -1;
    };
    let lhs = ptr_to_pyobject_ref_borrowed(left);
    let rhs = ptr_to_pyobject_ref_borrowed(right);
    rustpython_runtime::with_vm(|vm| match lhs.rich_compare_bool(&rhs, op, vm) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyObject_GetAttr(ob: *mut PyObject, attr_name: *mut PyObject) -> *mut PyObject {
    if ob.is_null() || attr_name.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    let name = ptr_to_pyobject_ref_borrowed(attr_name);
    rustpython_runtime::with_vm(|vm| {
        let Ok(name_str) = name.clone().try_into_value::<rustpython_vm::PyRef<PyStr>>(vm) else {
            set_vm_exception(vm.new_type_error("attribute name must be string"));
            return std::ptr::null_mut();
        };
        match obj.get_attr(&name_str, vm) {
            Ok(val) => pyobject_ref_to_ptr(val),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyObject_GetAttrString(
    ob: *mut PyObject,
    name: *const std::ffi::c_char,
) -> *mut PyObject {
    if ob.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    let name = match std::ffi::CStr::from_ptr(name).to_str() {
        Ok(name) => name,
        Err(_) => return std::ptr::null_mut(),
    };
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.get_attr(name, vm) {
        Ok(val) => pyobject_ref_to_ptr(val),
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyObject_SetAttr(
    ob: *mut PyObject,
    attr_name: *mut PyObject,
    value: *mut PyObject,
) -> c_int {
    if ob.is_null() || attr_name.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    let name = ptr_to_pyobject_ref_borrowed(attr_name);
    rustpython_runtime::with_vm(|vm| {
        let Ok(name_str) = name.clone().try_into_value::<rustpython_vm::PyRef<PyStr>>(vm) else {
            return -1;
        };
        if value.is_null() {
            match obj.del_attr(&name_str, vm) {
                Ok(()) => 0,
                Err(exc) => {
                    set_vm_exception(exc);
                    -1
                }
            }
        } else {
            let value = ptr_to_pyobject_ref_borrowed(value);
            match obj.set_attr(&name_str, value, vm) {
                Ok(()) => 0,
                Err(exc) => {
                    set_vm_exception(exc);
                    -1
                }
            }
        }
    })
}

#[inline]
pub unsafe fn PyObject_SetAttrString(
    ob: *mut PyObject,
    name: *const c_char,
    value: *mut PyObject,
) -> c_int {
    if ob.is_null() || name.is_null() {
        return -1;
    }
    let Ok(name) = std::ffi::CStr::from_ptr(name).to_str() else {
        return -1;
    };
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| {
        if value.is_null() {
            match obj.del_attr(name, vm) {
                Ok(()) => 0,
                Err(exc) => {
                    set_vm_exception(exc);
                    -1
                }
            }
        } else {
            let value = ptr_to_pyobject_ref_borrowed(value);
            match obj.set_attr(name, value, vm) {
                Ok(()) => 0,
                Err(exc) => {
                    set_vm_exception(exc);
                    -1
                }
            }
        }
    })
}

#[inline]
pub unsafe fn PyObject_GenericGetAttr(
    ob: *mut PyObject,
    attr_name: *mut PyObject,
) -> *mut PyObject {
    PyObject_GetAttr(ob, attr_name)
}

#[inline]
pub unsafe fn PyObject_GenericGetDict(
    ob: *mut PyObject,
    _closure: *mut c_void,
) -> *mut PyObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.dict() {
        Some(dict) => pyobject_ref_to_ptr(dict.into()),
        None => pyobject_ref_to_ptr(vm.ctx.new_dict().into()),
    })
}

#[inline]
pub unsafe fn PyObject_GenericSetDict(
    _ob: *mut PyObject,
    _value: *mut PyObject,
    _closure: *mut c_void,
) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyObject_ClearWeakRefs(_ob: *mut PyObject) {}

#[inline]
pub unsafe fn PyBytes_AS_STRING(obj: *mut PyObject) -> *mut c_char {
    crate::PyBytes_AsString(obj)
}

#[inline]
pub unsafe fn _PyBytes_Resize(obj: *mut *mut PyObject, newsize: Py_ssize_t) -> c_int {
    if obj.is_null() || (*obj).is_null() || newsize < 0 {
        return -1;
    }
    let original = ptr_to_pyobject_ref_borrowed(*obj);
    let Some(bytes) = original.downcast_ref::<rustpython_vm::builtins::PyBytes>() else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| {
        let mut data = bytes.as_bytes().to_vec();
        data.resize(newsize as usize, 0);
        *obj = pyobject_ref_to_ptr(vm.ctx.new_bytes(data).into());
    });
    0
}

#[inline]
pub unsafe fn PyCallable_Check(ob: *mut PyObject) -> c_int {
    if ob.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|_vm| obj.is_callable().into())
}

#[inline]
pub unsafe fn PyObject_Hash(ob: *mut PyObject) -> Py_hash_t {
    if ob.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.hash(vm) {
        Ok(hash) => hash as Py_hash_t,
        Err(_) => -1,
    })
}

#[inline]
pub unsafe fn PyObject_HashNotImplemented(_ob: *mut PyObject) -> Py_hash_t {
    -1
}

#[inline]
pub unsafe fn PyObject_IsTrue(ob: *mut PyObject) -> c_int {
    if ob.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match obj.is_true(vm) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    })
}

#[inline]
pub unsafe fn PyObject_Dir(ob: *mut PyObject) -> *mut PyObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| match vm.dir(Some(obj)) {
        Ok(dir) => pyobject_ref_to_ptr(PyList::into_ref(dir, &vm.ctx).into()),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn Py_None() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.none()))
}

#[inline]
pub unsafe fn Py_IsNone(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_None())
}

#[inline]
pub unsafe fn Py_NotImplemented() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.not_implemented()))
}

pub const Py_LT: c_int = 0;
pub const Py_LE: c_int = 1;
pub const Py_EQ: c_int = 2;
pub const Py_NE: c_int = 3;
pub const Py_GT: c_int = 4;
pub const Py_GE: c_int = 5;

pub const Py_TPFLAGS_HEAPTYPE: c_ulong = 1 << 9;
pub const Py_TPFLAGS_BASETYPE: c_ulong = 1 << 10;
pub const Py_TPFLAGS_READY: c_ulong = 1 << 12;
pub const Py_TPFLAGS_READYING: c_ulong = 1 << 13;
pub const Py_TPFLAGS_HAVE_GC: c_ulong = 1 << 14;
pub const Py_TPFLAGS_METHOD_DESCRIPTOR: c_ulong = 1 << 17;
pub const Py_TPFLAGS_VALID_VERSION_TAG: c_ulong = 1 << 19;
pub const Py_TPFLAGS_IS_ABSTRACT: c_ulong = 1 << 20;
pub const Py_TPFLAGS_LONG_SUBCLASS: c_ulong = 1 << 24;
pub const Py_TPFLAGS_LIST_SUBCLASS: c_ulong = 1 << 25;
pub const Py_TPFLAGS_TUPLE_SUBCLASS: c_ulong = 1 << 26;
pub const Py_TPFLAGS_BYTES_SUBCLASS: c_ulong = 1 << 27;
pub const Py_TPFLAGS_UNICODE_SUBCLASS: c_ulong = 1 << 28;
pub const Py_TPFLAGS_DICT_SUBCLASS: c_ulong = 1 << 29;
pub const Py_TPFLAGS_BASE_EXC_SUBCLASS: c_ulong = 1 << 30;
pub const Py_TPFLAGS_TYPE_SUBCLASS: c_ulong = 1 << 31;
pub const Py_TPFLAGS_DEFAULT: c_ulong = 0;
pub const Py_TPFLAGS_HAVE_FINALIZE: c_ulong = 1;
pub const Py_TPFLAGS_HAVE_VERSION_TAG: c_ulong = 1 << 18;
#[cfg(any(Py_3_12, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_HAVE_VECTORCALL: c_ulong = 1 << 11;
#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_SEQUENCE: c_ulong = 1 << 5;
#[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
pub const Py_TPFLAGS_MAPPING: c_ulong = 1 << 6;
#[cfg(Py_3_10)]
pub const Py_TPFLAGS_DISALLOW_INSTANTIATION: c_ulong = 1 << 7;
#[cfg(Py_3_10)]
pub const Py_TPFLAGS_IMMUTABLETYPE: c_ulong = 1 << 8;
#[cfg(Py_3_12)]
pub const Py_TPFLAGS_ITEMS_AT_END: c_ulong = 1 << 23;

#[cfg(Py_3_13)]
pub const Py_CONSTANT_NONE: c_uint = 0;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_FALSE: c_uint = 1;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_TRUE: c_uint = 2;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ELLIPSIS: c_uint = 3;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_NOT_IMPLEMENTED: c_uint = 4;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ZERO: c_uint = 5;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_ONE: c_uint = 6;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_STR: c_uint = 7;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_BYTES: c_uint = 8;
#[cfg(Py_3_13)]
pub const Py_CONSTANT_EMPTY_TUPLE: c_uint = 9;

#[inline]
pub unsafe fn PyType_HasFeature(ty: *mut PyTypeObject, feature: c_ulong) -> c_int {
    PyType_FastSubclass(ty, feature)
}

#[inline]
pub unsafe fn PyType_FastSubclass(ty: *mut PyTypeObject, feature: c_ulong) -> c_int {
    if ty.is_null() {
        return 0;
    }
    let ty = ptr_to_pyobject_ref_borrowed(ty as *mut PyObject);
    rustpython_runtime::with_vm(|vm| {
        let target: Option<PyObjectRef> = match feature {
            Py_TPFLAGS_LONG_SUBCLASS => Some(vm.ctx.types.int_type.to_owned().into()),
            Py_TPFLAGS_LIST_SUBCLASS => Some(vm.ctx.types.list_type.to_owned().into()),
            Py_TPFLAGS_TUPLE_SUBCLASS => Some(vm.ctx.types.tuple_type.to_owned().into()),
            Py_TPFLAGS_BYTES_SUBCLASS => Some(vm.ctx.types.bytes_type.to_owned().into()),
            Py_TPFLAGS_UNICODE_SUBCLASS => Some(vm.ctx.types.str_type.to_owned().into()),
            Py_TPFLAGS_DICT_SUBCLASS => Some(vm.ctx.types.dict_type.to_owned().into()),
            Py_TPFLAGS_BASE_EXC_SUBCLASS => {
                let exc: PyObjectRef = vm.ctx.exceptions.base_exception_type.to_owned().into();
                Some(exc)
            }
            Py_TPFLAGS_TYPE_SUBCLASS => Some(vm.ctx.types.type_type.to_owned().into()),
            _ => None,
        };
        match target {
            Some(target) => match ty.real_is_subclass(&target, vm) {
                Ok(true) => 1,
                _ => 0,
            },
            None => 0,
        }
    })
}

#[inline]
pub unsafe fn PyType_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        obj.class()
            .fast_issubclass(vm.ctx.types.type_type.as_object())
            .into()
    })
}

#[inline]
pub unsafe fn PyType_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        if let Ok(ty) = obj.try_to_ref::<PyType>(vm) {
            (ty.class().is(vm.ctx.types.type_type.as_object())).into()
        } else {
            0
        }
    })
}

#[inline]
pub unsafe fn PyType_FromSpec(spec: *mut PyType_Spec) -> *mut PyObject {
    if spec.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let name = if (*spec).name.is_null() {
            "<unnamed>"
        } else {
            ffi_name_to_static((*spec).name, "<invalid>")
        };
        let qual_name = name.rsplit('.').next().unwrap_or(name);

        let mut base: Option<PyTypeRef> = None;
        let mut attrs = vm.ctx.types.object_type.attributes.read().clone();
        attrs.clear();
        let mut slots = PyTypeSlots::new(ffi_name_to_static((*spec).name, "<unnamed>"), PyTypeFlags::DEFAULT);
        slots.basicsize = (*spec).basicsize.max(0) as usize;
        slots.itemsize = (*spec).itemsize.max(0) as usize;
        let mut metadata = HeapTypeMetadata::default();
        let mut method_defs: Vec<*mut methodobject::PyMethodDef> = Vec::new();

        let mut slot_ptr = (*spec).slots;
        while !slot_ptr.is_null() && (*slot_ptr).slot != 0 {
            match (*slot_ptr).slot {
                crate::Py_tp_base => {
                    let base_obj = ptr_to_pyobject_ref_borrowed((*slot_ptr).pfunc as *mut PyObject);
                    if let Ok(base_type) = base_obj.downcast::<PyType>() {
                        base = Some(base_type);
                    }
                }
                crate::Py_tp_doc => {
                    if !(*slot_ptr).pfunc.is_null() {
                        slots.doc = Some(ffi_name_to_static((*slot_ptr).pfunc.cast(), ""));
                    }
                }
                crate::Py_tp_methods => {
                    let mut def = (*slot_ptr).pfunc as *mut methodobject::PyMethodDef;
                    while !def.is_null() && !(*def).ml_name.is_null() {
                        method_defs.push(def);
                        def = def.add(1);
                    }
                }
                crate::Py_tp_getset => {
                    let mut def = (*slot_ptr).pfunc as *mut crate::descrobject::PyGetSetDef;
                    while !def.is_null() && !(*def).name.is_null() {
                        attrs.insert(
                            vm.ctx.intern_str(ffi_name_to_static((*def).name, "<property>")),
                            build_getter_property(def, vm),
                        );
                        def = def.add(1);
                    }
                }
                crate::Py_tp_new => {
                    metadata.tp_new = (*slot_ptr).pfunc as usize;
                    slots.new.store(Some(heap_tp_new_wrapper));
                }
                crate::Py_tp_init => {
                    metadata.tp_init = (*slot_ptr).pfunc as usize;
                    slots.init.store(Some(heap_tp_init_wrapper));
                }
                crate::Py_tp_call => metadata.tp_call = (*slot_ptr).pfunc as usize,
                crate::Py_mp_subscript => metadata.mp_subscript = (*slot_ptr).pfunc as usize,
                crate::Py_mp_ass_subscript => metadata.mp_ass_subscript = (*slot_ptr).pfunc as usize,
                crate::Py_nb_add => metadata.nb_add = (*slot_ptr).pfunc as usize,
                crate::Py_sq_length => metadata.sq_length = (*slot_ptr).pfunc as usize,
                crate::Py_sq_item => metadata.sq_item = (*slot_ptr).pfunc as usize,
                crate::Py_sq_ass_item => metadata.sq_ass_item = (*slot_ptr).pfunc as usize,
                crate::Py_sq_contains => metadata.sq_contains = (*slot_ptr).pfunc as usize,
                crate::Py_sq_concat => metadata.sq_concat = (*slot_ptr).pfunc as usize,
                crate::Py_sq_repeat => metadata.sq_repeat = (*slot_ptr).pfunc as usize,
                crate::Py_sq_inplace_concat => {
                    metadata.sq_inplace_concat = (*slot_ptr).pfunc as usize
                }
                crate::Py_sq_inplace_repeat => {
                    metadata.sq_inplace_repeat = (*slot_ptr).pfunc as usize
                }
                _ => {}
            }
            slot_ptr = slot_ptr.add(1);
        }

        if (*spec).flags as c_ulong & Py_TPFLAGS_BASETYPE != 0 {
            slots.flags |= PyTypeFlags::BASETYPE;
        }
        if (*spec).flags as c_ulong & (1 << 8) != 0 {
            slots.flags |= PyTypeFlags::IMMUTABLETYPE;
        }
        if (*spec).flags as c_ulong & (1 << 6) != 0 {
            slots.flags |= PyTypeFlags::MAPPING;
        }
        if (*spec).flags as c_ulong & (1 << 5) != 0 {
            slots.flags |= PyTypeFlags::SEQUENCE;
        }

        let module_name = qual_name
            .rsplit_once('.')
            .map(|(module, _)| module)
            .unwrap_or("builtins");
        attrs.insert(
            vm.ctx.intern_str("__module__"),
            vm.ctx.new_str(module_name).into(),
        );
        if let Some(doc) = slots.doc {
            attrs.insert(
                vm.ctx.intern_str("__doc__"),
                vm.ctx.new_str(doc_from_internal_doc(qual_name, doc)).into(),
            );
        }

        let Some(base) = base else {
            return std::ptr::null_mut();
        };
        match PyType::new_heap(
            qual_name,
            vec![base],
            attrs,
            slots,
            vm.ctx.types.type_type.to_owned(),
            &vm.ctx,
        ) {
            Ok(ty) => {
                let class: &'static rustpython_vm::Py<PyType> =
                    unsafe { std::mem::transmute::<&rustpython_vm::Py<PyType>, &'static rustpython_vm::Py<PyType>>(&*ty) };
                if metadata.tp_new != 0 {
                    ty.slots.new.store(Some(heap_tp_new_wrapper));
                }
                if metadata.tp_init != 0 {
                    ty.slots.init.store(Some(heap_tp_init_wrapper));
                }
                if metadata.tp_call != 0 {
                    ty.slots.call.store(Some(heap_tp_call_wrapper));
                }
                for def in method_defs {
                    let name = ffi_name_to_static((*def).ml_name, "<method>");
                    let method = unsafe {
                        methodobject::build_rustpython_class_method(def, class, vm)
                    };
                    ty.set_attr(vm.ctx.intern_str(name), method);
                }
                if metadata.mp_subscript != 0 {
                    ty.slots
                        .as_mapping
                        .subscript
                        .store(Some(heap_mapping_getitem_wrapper));
                }
                if metadata.mp_ass_subscript != 0 {
                    ty.slots
                        .as_mapping
                        .ass_subscript
                        .store(Some(heap_mapping_setitem_wrapper));
                }
                if metadata.nb_add != 0 {
                    ty.slots.as_number.add.store(Some(heap_nb_add_wrapper));
                }
                if metadata.sq_length != 0 {
                    ty.slots
                        .as_sequence
                        .length
                        .store(Some(heap_sq_length_wrapper));
                }
                if metadata.sq_item != 0 {
                    ty.slots.as_sequence.item.store(Some(heap_sq_item_wrapper));
                }
                if metadata.sq_ass_item != 0 {
                    ty.slots
                        .as_sequence
                        .ass_item
                        .store(Some(heap_sq_ass_item_wrapper));
                }
                if metadata.sq_contains != 0 {
                    ty.slots
                        .as_sequence
                        .contains
                        .store(Some(heap_sq_contains_wrapper));
                }
                if metadata.sq_concat != 0 {
                    ty.slots
                        .as_sequence
                        .concat
                        .store(Some(heap_sq_concat_wrapper));
                }
                if metadata.sq_repeat != 0 {
                    ty.slots
                        .as_sequence
                        .repeat
                        .store(Some(heap_sq_repeat_wrapper));
                }
                if metadata.sq_inplace_concat != 0 {
                    ty.slots
                        .as_sequence
                        .inplace_concat
                        .store(Some(heap_sq_inplace_concat_wrapper));
                }
                if metadata.sq_inplace_repeat != 0 {
                    ty.slots
                        .as_sequence
                        .inplace_repeat
                        .store(Some(heap_sq_inplace_repeat_wrapper));
                }
                let type_obj: PyObjectRef = ty.into();
                let type_ptr = pyobject_ref_as_ptr(&type_obj) as *mut PyTypeObject;
                heap_type_registry()
                    .lock()
                    .unwrap()
                    .insert(type_ptr as usize, metadata);
                pyobject_ref_to_ptr(type_obj)
            }
            Err(err) => {
                let exc = vm.new_type_error(err);
                crate::pyerrors::set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyType_GetSlot(ty: *mut PyTypeObject, slot: c_int) -> *mut c_void {
    if ty.is_null() {
        return std::ptr::null_mut();
    }
    if let Some(metadata) = heap_type_registry().lock().unwrap().get(&(ty as usize)).copied() {
        return match slot {
            crate::Py_tp_new => metadata.tp_new as *mut c_void,
            crate::Py_tp_init => metadata.tp_init as *mut c_void,
            _ => std::ptr::null_mut(),
        };
    }
    rustpython_runtime::with_vm(|vm| match slot {
        crate::Py_tp_new => {
            let ty_obj = unsafe { ptr_to_pyobject_ref_borrowed(ty as *mut PyObject) };
            if ty == pyobject_ref_as_ptr(&vm.ctx.types.object_type.to_owned().into()) as *mut PyTypeObject {
                builtin_object_tp_new as *mut c_void
            } else if ty == pyobject_ref_as_ptr(&vm.ctx.types.set_type.to_owned().into()) as *mut PyTypeObject {
                builtin_set_tp_new as *mut c_void
            } else if ty == pyobject_ref_as_ptr(&vm.ctx.types.dict_type.to_owned().into()) as *mut PyTypeObject {
                builtin_dict_tp_new as *mut c_void
            } else if ty_obj
                .downcast::<PyType>()
                .map(|cls| cls.fast_issubclass(vm.ctx.exceptions.base_exception_type))
                .unwrap_or(false)
            {
                builtin_exception_tp_new as *mut c_void
            } else {
                std::ptr::null_mut()
            }
        }
        _ => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyType_GenericAlloc(
    subtype: *mut PyTypeObject,
    nitems: Py_ssize_t,
) -> *mut PyObject {
    let _ = (subtype, nitems);
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_HASH_CUTOFF() -> Py_hash_t {
    0
}
