use crate::pyport::{Py_hash_t, Py_ssize_t};
use crate::rustpython_runtime;
use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use std::ptr::NonNull;

use rustpython_vm::builtins::{PyList, PyStr, PyType};
use rustpython_vm::types::PyComparisonOp;
use rustpython_vm::{AsObject, PyObjectRef, PyPayload};

#[repr(C)]
#[derive(Debug)]
pub struct PyObject {
    pub(crate) _opaque: [u8; 0],
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

pub const PyObject_HEAD_INIT: PyObject = PyObject { _opaque: [] };

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
        Err(_) => std::ptr::null_mut(),
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
        Err(_) => -1,
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
            return std::ptr::null_mut();
        };
        match obj.get_attr(&name_str, vm) {
            Ok(val) => pyobject_ref_to_ptr(val),
            Err(_) => std::ptr::null_mut(),
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
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyObject_SetAttr(
    _ob: *mut PyObject,
    _attr_name: *mut PyObject,
    _value: *mut PyObject,
) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyObject_SetAttrString(
    _ob: *mut PyObject,
    _name: *const c_char,
    _value: *mut PyObject,
) -> c_int {
    -1
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
pub unsafe fn PyBytes_AS_STRING(_obj: *mut PyObject) -> *mut c_char {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn _PyBytes_Resize(_obj: *mut *mut PyObject, _newsize: Py_ssize_t) -> c_int {
    -1
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
        if let Ok(ty) = obj.try_to_ref::<PyType>(vm) {
            return match ty
                .as_object()
                .is_subclass(vm.ctx.types.type_type.as_object(), vm)
            {
                Ok(true) => 1,
                _ => 0,
            };
        }
        0
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
pub unsafe fn PyType_FromSpec(_spec: *mut PyType_Spec) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyType_GetSlot(_ty: *mut PyTypeObject, _slot: c_int) -> *mut c_void {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyType_GenericAlloc(
    _subtype: *mut PyTypeObject,
    _nitems: Py_ssize_t,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_HASH_CUTOFF() -> Py_hash_t {
    0
}
