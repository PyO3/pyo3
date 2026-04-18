use crate::object::*;
use crate::pyerrors::PyErr_GetRaisedException;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyBaseException, PyStr, PyType};
use rustpython_vm::function::{FuncArgs, PyMethodDef as RpMethodDef, PyMethodFlags as RpMethodFlags};
use rustpython_vm::{AsObject, PyObjectRef};
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::{Mutex, OnceLock};
use std::{mem, ptr};

pub static mut PyCFunction_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[cfg(all(Py_3_9, not(Py_LIMITED_API), not(GraalPy)))]
pub struct PyCFunctionObject {
    pub ob_base: PyObject,
    pub m_ml: *mut PyMethodDef,
    pub m_self: *mut PyObject,
    pub m_module: *mut PyObject,
    pub m_weakreflist: *mut PyObject,
    #[cfg(not(PyPy))]
    pub vectorcall: Option<crate::vectorcallfunc>,
}

pub type PyCFunction =
    unsafe extern "C" fn(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
pub type PyCFunctionFast = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
) -> *mut PyObject;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[deprecated(note = "renamed to `PyCFunctionFast`")]
pub type _PyCFunctionFast = PyCFunctionFast;

pub type PyCFunctionWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
pub type PyCFunctionFastWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *const *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[deprecated(note = "renamed to `PyCFunctionFastWithKeywords`")]
pub type _PyCFunctionFastWithKeywords = PyCFunctionFastWithKeywords;

#[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
pub type PyCMethod = unsafe extern "C" fn(
    slf: *mut PyObject,
    defining_class: *mut PyTypeObject,
    args: *const *mut PyObject,
    nargs: crate::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PyMethodDef {
    pub ml_name: *const c_char,
    pub ml_meth: PyMethodDefPointer,
    pub ml_flags: c_int,
    pub ml_doc: *const c_char,
}

impl PyMethodDef {
    pub const fn zeroed() -> PyMethodDef {
        PyMethodDef {
            ml_name: ptr::null(),
            ml_meth: PyMethodDefPointer {
                Void: ptr::null_mut(),
            },
            ml_flags: 0,
            ml_doc: ptr::null(),
        }
    }
}

impl Default for PyMethodDef {
    fn default() -> PyMethodDef {
        PyMethodDef::zeroed()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq)]
pub union PyMethodDefPointer {
    pub PyCFunction: PyCFunction,
    pub PyCFunctionWithKeywords: PyCFunctionWithKeywords,
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[deprecated(note = "renamed to `PyCFunctionFast`")]
    pub _PyCFunctionFast: PyCFunctionFast,
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub PyCFunctionFast: PyCFunctionFast,
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[deprecated(note = "renamed to `PyCFunctionFastWithKeywords`")]
    pub _PyCFunctionFastWithKeywords: PyCFunctionFastWithKeywords,
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub PyCFunctionFastWithKeywords: PyCFunctionFastWithKeywords,
    #[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
    pub PyCMethod: PyCMethod,
    Void: *mut c_void,
}

impl PyMethodDefPointer {
    pub fn as_ptr(&self) -> *mut c_void {
        unsafe { self.Void }
    }

    pub fn is_null(&self) -> bool {
        self.as_ptr().is_null()
    }

    pub const fn zeroed() -> PyMethodDefPointer {
        PyMethodDefPointer {
            Void: ptr::null_mut(),
        }
    }
}

impl PartialEq for PyMethodDefPointer {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.Void == other.Void }
    }
}

impl std::fmt::Pointer for PyMethodDefPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = unsafe { self.Void };
        std::fmt::Pointer::fmt(&ptr, f)
    }
}

const _: () =
    assert!(mem::size_of::<PyMethodDefPointer>() == mem::size_of::<Option<extern "C" fn()>>());

pub const METH_VARARGS: c_int = 0x0001;
pub const METH_KEYWORDS: c_int = 0x0002;
pub const METH_NOARGS: c_int = 0x0004;
pub const METH_O: c_int = 0x0008;
pub const METH_CLASS: c_int = 0x0010;
pub const METH_STATIC: c_int = 0x0020;
pub const METH_COEXIST: c_int = 0x0040;
#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
pub const METH_FASTCALL: c_int = 0x0080;
#[cfg(all(Py_3_9, not(Py_LIMITED_API)))]
pub const METH_METHOD: c_int = 0x0200;

#[derive(Copy, Clone)]
struct MethodMetadata {
    name: &'static str,
    method_def: usize,
    slf: usize,
    flags: c_int,
}

const PYO3_METHOD_DEF_ATTR: &str = "__pyo3_method_def_ptr__";
const PYO3_METHOD_SELF_ATTR: &str = "__pyo3_method_self_ptr__";
const PYO3_METHOD_FLAGS_ATTR: &str = "__pyo3_method_flags__";

fn method_metadata_registry() -> &'static Mutex<HashMap<usize, MethodMetadata>> {
    static REGISTRY: OnceLock<Mutex<HashMap<usize, MethodMetadata>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lookup_method_metadata(obj: &PyObjectRef) -> Option<MethodMetadata> {
    let obj_ptr = pyobject_ref_as_ptr(obj) as usize;
    rustpython_runtime::with_vm(|vm| {
        let attrs_metadata = obj
            .get_attr(PYO3_METHOD_DEF_ATTR, vm)
            .ok()
            .and_then(|value| value.try_to_value::<isize>(vm).ok())
            .zip(
                obj.get_attr(PYO3_METHOD_SELF_ATTR, vm)
                    .ok()
                    .and_then(|value| value.try_to_value::<isize>(vm).ok()),
            )
            .zip(
                obj.get_attr(PYO3_METHOD_FLAGS_ATTR, vm)
                    .ok()
                    .and_then(|value| value.try_to_value::<i32>(vm).ok()),
            )
            .map(|((method_def, slf), flags)| MethodMetadata {
                name: "<attr-backed>",
                method_def: method_def as usize,
                slf: slf as usize,
                flags,
            });

        if attrs_metadata.is_some() {
            return attrs_metadata;
        }

        let metadata = method_metadata_registry()
            .lock()
            .unwrap()
            .get(&obj_ptr)
            .copied()?;
        let current_name = obj
            .get_attr("__name__", vm)
            .ok()
            .and_then(|value| value.str(vm).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| obj.class().slot_name().to_owned());
        if current_name == metadata.name {
            Some(metadata)
        } else {
            method_metadata_registry().lock().unwrap().remove(&obj_ptr);
            None
        }
    })
}

pub(crate) unsafe fn call_with_original_args(
    callable: &PyObjectRef,
    args: *mut PyObject,
    kwargs: *mut PyObject,
) -> Option<*mut PyObject> {
    let metadata = lookup_method_metadata(callable)?;
    let method_def = metadata.method_def as *mut PyMethodDef;
    if method_def.is_null() {
        return None;
    }
    let slf = metadata.slf as *mut PyObject;
    let flags = metadata.flags;
    let method = unsafe { &*method_def };

    if flags & METH_VARARGS != 0 && flags & METH_KEYWORDS != 0 {
        return Some(unsafe { (method.ml_meth.PyCFunctionWithKeywords)(slf, args, kwargs) });
    }

    if flags & METH_NOARGS != 0 {
        return Some(unsafe { (method.ml_meth.PyCFunction)(slf, std::ptr::null_mut()) });
    }

    None
}

const SIGNATURE_END_MARKER: &str = ")\n--\n\n";

fn find_signature<'a>(name: &str, doc: &'a str) -> Option<&'a str> {
    let name = name.rsplit('.').next().unwrap_or(name);
    let doc = doc.strip_prefix(name)?;
    doc.starts_with('(').then_some(doc)
}

fn text_signature_from_internal_doc<'a>(name: &str, internal_doc: &'a str) -> Option<&'a str> {
    find_signature(name, internal_doc)
        .and_then(|doc| doc.find(SIGNATURE_END_MARKER).map(|index| &doc[..=index]))
}

fn doc_from_internal_doc<'a>(name: &str, internal_doc: &'a str) -> &'a str {
    if let Some(doc_without_sig) = find_signature(name, internal_doc) {
        if let Some(sig_end_pos) = doc_without_sig.find(SIGNATURE_END_MARKER) {
            return &doc_without_sig[sig_end_pos + SIGNATURE_END_MARKER.len()..];
        }
    }
    internal_doc
}

fn current_method_doc(obj: &PyObjectRef) -> Option<(&'static str, &'static str)> {
    let metadata = lookup_method_metadata(obj)?;
    if metadata.method_def == 0 {
        return None;
    }
    let method_def = unsafe { &*(metadata.method_def as *const PyMethodDef) };
    if method_def.ml_doc.is_null() {
        return None;
    }
    let name = ffi_name_to_static(method_def.ml_name, "<unnamed>");
    let raw_doc = ffi_name_to_static(method_def.ml_doc, "");
    Some((name, raw_doc))
}

fn descriptor_fallback(
    vm: &rustpython_vm::VirtualMachine,
    descriptor: &PyObjectRef,
    obj: PyObjectRef,
) -> rustpython_vm::PyResult {
    vm.call_method(descriptor, "__get__", (obj.clone(), obj.class().to_owned()))
}

fn method_name_from_object(
    vm: &rustpython_vm::VirtualMachine,
    obj: &PyObjectRef,
) -> Option<String> {
    obj.get_attr("__name__", vm)
        .ok()
        .and_then(|name| name.downcast_ref::<PyStr>().map(|s| AsRef::<str>::as_ref(s).to_owned()))
}

fn normalize_doc_object(
    vm: &rustpython_vm::VirtualMachine,
    obj: &PyObjectRef,
    value: PyObjectRef,
) -> PyObjectRef {
    let Some(raw_doc) = value.downcast_ref::<PyStr>().map(|s| AsRef::<str>::as_ref(s)) else {
        return value;
    };
    let Some(name) = method_name_from_object(vm, obj) else {
        return value;
    };
    vm.ctx.new_str(doc_from_internal_doc(&name, raw_doc)).into()
}

fn normalize_text_signature_object(
    vm: &rustpython_vm::VirtualMachine,
    obj: &PyObjectRef,
    value: PyObjectRef,
) -> PyObjectRef {
    let Some(raw_doc) = value.downcast_ref::<PyStr>().map(|s| AsRef::<str>::as_ref(s)) else {
        return value;
    };
    let Some(name) = method_name_from_object(vm, obj) else {
        return value;
    };
    match text_signature_from_internal_doc(&name, raw_doc) {
        Some(sig) => vm.ctx.new_str(sig).into(),
        None => vm.ctx.none(),
    }
}

fn install_doc_descriptors(
    vm: &rustpython_vm::VirtualMachine,
    class: &'static rustpython_vm::Py<PyType>,
) {
    let doc_name = vm.ctx.intern_str("__doc__");
    let textsig_name = vm.ctx.intern_str("__text_signature__");

    let original_doc = class.get_direct_attr(doc_name).unwrap();
    let original_textsig = class.get_direct_attr(textsig_name).unwrap();

    class.set_attr(
        doc_name,
        vm.ctx
            .new_readonly_getset(
                "__doc__",
                class,
                move |obj: PyObjectRef, vm: &rustpython_vm::VirtualMachine| {
                    if let Some((name, raw_doc)) = current_method_doc(&obj) {
                        return Ok(vm.ctx.new_str(doc_from_internal_doc(name, raw_doc)).into());
                    }
                    descriptor_fallback(vm, &original_doc, obj.clone())
                        .map(|value| normalize_doc_object(vm, &obj, value))
                },
            )
            .into(),
    );

    class.set_attr(
        textsig_name,
        vm.ctx
            .new_readonly_getset(
                "__text_signature__",
                class,
                move |obj: PyObjectRef, vm: &rustpython_vm::VirtualMachine| {
                    if let Some((name, raw_doc)) = current_method_doc(&obj) {
                        return Ok(match text_signature_from_internal_doc(name, raw_doc) {
                            Some(sig) => vm.ctx.new_str(sig).into(),
                            None => vm.ctx.none(),
                        });
                    }
                    descriptor_fallback(vm, &original_textsig, obj.clone())
                        .map(|value| normalize_text_signature_object(vm, &obj, value))
                },
            )
            .into(),
    );
}

pub(crate) fn init_builtin_function_descriptors(vm: &rustpython_vm::VirtualMachine) {
    static INITIALIZED: OnceLock<()> = OnceLock::new();
    INITIALIZED.get_or_init(|| {
        install_doc_descriptors(vm, vm.ctx.types.builtin_function_or_method_type);
        install_doc_descriptors(vm, vm.ctx.types.method_descriptor_type);
    });
}

#[inline]
pub unsafe fn PyCFunction_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| obj.class().is(vm.ctx.types.builtin_function_or_method_type).into())
}

#[inline]
pub unsafe fn PyCFunction_Check(op: *mut PyObject) -> c_int {
    PyCFunction_CheckExact(op)
}

#[inline]
pub unsafe fn PyCFunction_GetFunction(f: *mut PyObject) -> Option<PyCFunction> {
    let metadata = lookup_method_metadata(&ptr_to_pyobject_ref_borrowed(f))?;
    if metadata.method_def == 0 {
        return None;
    }
    let method_def = &*(metadata.method_def as *mut PyMethodDef);
    if method_def.ml_flags & (METH_VARARGS | METH_KEYWORDS | METH_FASTCALL) == 0 {
        Some(method_def.ml_meth.PyCFunction)
    } else {
        None
    }
}

#[inline]
pub unsafe fn PyCFunction_GetSelf(f: *mut PyObject) -> *mut PyObject {
    lookup_method_metadata(&ptr_to_pyobject_ref_borrowed(f))
        .map(|metadata| metadata.slf as *mut PyObject)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyCFunction_GetFlags(f: *mut PyObject) -> c_int {
    lookup_method_metadata(&ptr_to_pyobject_ref_borrowed(f))
        .map(|metadata| metadata.flags)
        .unwrap_or(0)
}

#[cfg(not(Py_3_13))]
#[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
#[inline]
pub unsafe fn PyCFunction_Call(
    f: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    crate::PyObject_Call(f, args, kwds)
}

fn ffi_name_to_static(ptr: *const c_char, default: &'static str) -> &'static str {
    if ptr.is_null() {
        return default;
    }
    let owned = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
        .into_boxed_str();
    Box::leak(owned)
}

unsafe fn fetch_current_exception(vm: &rustpython_vm::VirtualMachine) -> rustpython_vm::builtins::PyBaseExceptionRef {
    let raised = PyErr_GetRaisedException();
    if raised.is_null() {
        return vm.new_system_error("PyCFunction returned NULL without setting an exception");
    }
    match ptr_to_pyobject_ref_owned(raised).downcast::<PyBaseException>() {
        Ok(exc) => exc,
        Err(obj) => vm.new_system_error(format!(
            "PyCFunction set a non-exception object: {}",
            obj.class().name()
        )),
    }
}

unsafe fn call_varargs(
    vm: &rustpython_vm::VirtualMachine,
    method_def: *mut PyMethodDef,
    slf: *mut PyObject,
    args: FuncArgs,
) -> rustpython_vm::PyResult {
    let tuple = vm.ctx.new_tuple(args.args);
    let tuple_obj: PyObjectRef = tuple.into();
    let kwargs = if args.kwargs.is_empty() {
        None
    } else {
        let dict = vm.ctx.new_dict();
        for (key, value) in args.kwargs {
            if let Err(exc) = dict.set_item(key.as_str(), value, vm) {
                return Err(exc);
            }
        }
        Some(dict)
    };
    let kwargs_obj = kwargs.as_ref().map(|dict| -> PyObjectRef { dict.clone().into() });
    let method = &*method_def;
    let result = (method.ml_meth.PyCFunctionWithKeywords)(
        slf,
        pyobject_ref_as_ptr(&tuple_obj),
        kwargs_obj
            .as_ref()
            .map(pyobject_ref_as_ptr)
            .unwrap_or(std::ptr::null_mut()),
    );
    if result.is_null() {
        Err(fetch_current_exception(vm))
    } else {
        Ok(ptr_to_pyobject_ref_owned(result))
    }
}

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
unsafe fn call_fastcall(
    vm: &rustpython_vm::VirtualMachine,
    method_def: *mut PyMethodDef,
    slf: *mut PyObject,
    args: FuncArgs,
) -> rustpython_vm::PyResult {
    let positional_len = args.args.len();
    let mut owned = args.args;
    let mut keyword_names = Vec::with_capacity(args.kwargs.len());
    for (name, value) in args.kwargs {
        keyword_names.push(vm.ctx.new_str(name));
        owned.push(value);
    }
    let mut raw_args = owned
        .iter()
        .map(pyobject_ref_as_ptr)
        .collect::<Vec<*mut PyObject>>();
    let keyword_names_tuple = if keyword_names.is_empty() {
        None
    } else {
        Some(vm.ctx.new_tuple(
            keyword_names
                .into_iter()
            .map(|name| name.into())
            .collect::<Vec<PyObjectRef>>(),
        ))
    };
    let keyword_names_tuple_obj = keyword_names_tuple
        .as_ref()
        .map(|tuple| -> PyObjectRef { tuple.clone().into() });
    let method = &*method_def;
    let result = (method.ml_meth.PyCFunctionFastWithKeywords)(
        slf,
        raw_args.as_mut_ptr().cast_const(),
        positional_len as Py_ssize_t,
        keyword_names_tuple_obj
            .as_ref()
            .map(pyobject_ref_as_ptr)
            .unwrap_or(std::ptr::null_mut()),
    );
    if result.is_null() {
        Err(fetch_current_exception(vm))
    } else {
        Ok(ptr_to_pyobject_ref_owned(result))
    }
}

unsafe fn call_noargs(
    vm: &rustpython_vm::VirtualMachine,
    method_def: *mut PyMethodDef,
    slf: *mut PyObject,
    args: FuncArgs,
) -> rustpython_vm::PyResult {
    if !args.args.is_empty() || !args.kwargs.is_empty() {
        return Err(vm.new_type_error("this builtin function takes no arguments"));
    }
    let method = &*method_def;
    let result = (method.ml_meth.PyCFunction)(slf, std::ptr::null_mut());
    if result.is_null() {
        Err(fetch_current_exception(vm))
    } else {
        Ok(ptr_to_pyobject_ref_owned(result))
    }
}

unsafe fn call_ffi_method(
    vm: &rustpython_vm::VirtualMachine,
    method_def: *mut PyMethodDef,
    slf: *mut PyObject,
    args: FuncArgs,
) -> rustpython_vm::PyResult {
    let flags = (*method_def).ml_flags;
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    if flags & METH_FASTCALL != 0 && flags & METH_KEYWORDS != 0 {
        return call_fastcall(vm, method_def, slf, args);
    }
    if flags & METH_VARARGS != 0 && flags & METH_KEYWORDS != 0 {
        return call_varargs(vm, method_def, slf, args);
    }
    if flags & METH_NOARGS != 0 {
        return call_noargs(vm, method_def, slf, args);
    }
    Err(vm.new_system_error(format!(
        "unsupported PyCFunction flags: 0x{:x}",
        flags
    )))
}

fn ffi_method_flags(flags: c_int) -> RpMethodFlags {
    if flags & METH_CLASS != 0 {
        RpMethodFlags::CLASS
    } else if flags & METH_STATIC != 0 {
        RpMethodFlags::STATIC
    } else {
        RpMethodFlags::METHOD
    }
}

pub(crate) unsafe fn build_rustpython_class_method(
    ml: *mut PyMethodDef,
    class: &'static rustpython_vm::Py<PyType>,
    vm: &rustpython_vm::VirtualMachine,
) -> PyObjectRef {
    let name = ffi_name_to_static((*ml).ml_name, "<unnamed>");
    let doc = if (*ml).ml_doc.is_null() {
        None
    } else {
        Some(ffi_name_to_static((*ml).ml_doc, ""))
    };
    let flags = ffi_method_flags((*ml).ml_flags);
    let method_ptr = ml as usize;
    let method_def = Box::leak(Box::new(RpMethodDef {
        name,
        func: Box::leak(Box::new(move |vm: &rustpython_vm::VirtualMachine, mut args: FuncArgs| {
            let slf = if flags.contains(RpMethodFlags::STATIC) {
                std::ptr::null_mut()
            } else {
                let Some(first) = args.args.first().cloned() else {
                    return Err(vm.new_type_error(format!(
                        "missing bound receiver for method {name}"
                    )));
                };
                args.args.remove(0);
                pyobject_ref_as_ptr(&first)
            };
            let method_def = method_ptr as *mut PyMethodDef;
            unsafe { call_ffi_method(vm, method_def, slf, args) }
        })),
        flags,
        doc,
    }));
    let obj = method_def.to_proper_method(class, &vm.ctx);
    method_metadata_registry().lock().unwrap().insert(
        pyobject_ref_as_ptr(&obj) as usize,
        MethodMetadata {
            name,
            method_def: ml as usize,
            slf: 0,
            flags: (*ml).ml_flags,
        },
    );
    let _ = obj.set_attr(PYO3_METHOD_DEF_ATTR, vm.ctx.new_int(ml as isize), vm);
    let _ = obj.set_attr(PYO3_METHOD_SELF_ATTR, vm.ctx.new_int(0), vm);
    let _ = obj.set_attr(PYO3_METHOD_FLAGS_ATTR, vm.ctx.new_int((*ml).ml_flags), vm);
    obj
}

unsafe fn build_rustpython_function(
    ml: *mut PyMethodDef,
    slf: *mut PyObject,
    module: *mut PyObject,
) -> *mut PyObject {
    if ml.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let slf_obj = (!slf.is_null()).then(|| ptr_to_pyobject_ref_borrowed(slf));
        let slf_ptr = slf_obj
            .as_ref()
            .map(pyobject_ref_as_ptr)
            .unwrap_or(std::ptr::null_mut());
        let module_name = if module.is_null() {
            None
        } else {
            ptr_to_pyobject_ref_borrowed(module)
                .downcast_ref::<PyStr>()
                .map(|name| vm.ctx.intern_str(AsRef::<str>::as_ref(name)))
        };

        let name = ffi_name_to_static((*ml).ml_name, "<unnamed>");
        let doc = if (*ml).ml_doc.is_null() {
            None
        } else {
            Some(ffi_name_to_static((*ml).ml_doc, ""))
        };
        let flags = (*ml).ml_flags;
        let method_ptr = ml as usize;
        let method_def = Box::leak(Box::new(RpMethodDef {
            name,
            func: Box::leak(Box::new(move |vm: &rustpython_vm::VirtualMachine, args: FuncArgs| {
                let slf = slf_obj
                    .as_ref()
                    .map(pyobject_ref_as_ptr)
                    .unwrap_or(std::ptr::null_mut());
                let method_def = method_ptr as *mut PyMethodDef;
                // SAFETY: `method_ptr` points at a leaked/static FFI method definition supplied by PyO3.
                unsafe { call_ffi_method(vm, method_def, slf, args) }
            })),
            flags: RpMethodFlags::EMPTY,
            doc,
        }));

        let function = if let Some(module_name) = module_name {
            method_def.to_function().with_module(module_name).into_ref(&vm.ctx)
        } else {
            method_def.build_function(&vm.ctx)
        };
        let obj: PyObjectRef = function.into();
        method_metadata_registry().lock().unwrap().insert(
            pyobject_ref_as_ptr(&obj) as usize,
            MethodMetadata {
                name,
                method_def: ml as usize,
                slf: slf_ptr as usize,
                flags,
            },
        );
        if let Some(doc) = doc {
            let _ = obj.set_attr("__doc__", vm.ctx.new_str(doc), vm);
        }
        let _ = obj.set_attr(PYO3_METHOD_DEF_ATTR, vm.ctx.new_int(ml as isize), vm);
        let _ = obj.set_attr(PYO3_METHOD_SELF_ATTR, vm.ctx.new_int(slf_ptr as isize), vm);
        let _ = obj.set_attr(PYO3_METHOD_FLAGS_ATTR, vm.ctx.new_int(flags), vm);
        pyobject_ref_to_ptr(obj)
    })
}

#[inline]
pub unsafe fn PyCFunction_New(ml: *mut PyMethodDef, slf: *mut PyObject) -> *mut PyObject {
    PyCFunction_NewEx(ml, slf, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyCFunction_NewEx(
    ml: *mut PyMethodDef,
    slf: *mut PyObject,
    module: *mut PyObject,
) -> *mut PyObject {
    build_rustpython_function(ml, slf, module)
}

#[inline]
pub unsafe fn PyCMethod_New(
    ml: *mut PyMethodDef,
    slf: *mut PyObject,
    module: *mut PyObject,
    _cls: *mut PyTypeObject,
) -> *mut PyObject {
    build_rustpython_function(ml, slf, module)
}

#[cfg(not(Py_3_9))]
#[inline]
pub unsafe fn PyCFunction_ClearFreeList() -> c_int {
    0
}
