//! `PyClass` and related traits.
use crate::{
    class::impl_::{fallback_new, tp_dealloc, PyClassImpl},
    ffi,
    pyclass_slots::{PyClassDict, PyClassWeakRef},
    PyCell, PyErr, PyMethodDefType, PyNativeType, PyResult, PyTypeInfo, Python,
};
use std::{
    convert::TryInto,
    ffi::CString,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr,
};

/// If `PyClass` is implemented for `T`, then we can use `T` in the Python world,
/// via `PyCell`.
///
/// The `#[pyclass]` attribute automatically implements this trait for your Rust struct,
/// so you don't have to use this trait directly.
pub trait PyClass:
    PyTypeInfo<AsRefTarget = PyCell<Self>> + PyClassImpl<Layout = PyCell<Self>>
{
    /// Specify this class has `#[pyclass(dict)]` or not.
    type Dict: PyClassDict;
    /// Specify this class has `#[pyclass(weakref)]` or not.
    type WeakRef: PyClassWeakRef;
    /// The closest native ancestor. This is `PyAny` by default, and when you declare
    /// `#[pyclass(extends=PyDict)]`, it's `PyDict`.
    type BaseNativeType: PyTypeInfo + PyNativeType;
}

/// For collecting slot items.
#[derive(Default)]
struct TypeSlots(Vec<ffi::PyType_Slot>);

impl TypeSlots {
    fn push(&mut self, slot: c_int, pfunc: *mut c_void) {
        self.0.push(ffi::PyType_Slot { slot, pfunc });
    }
}

fn tp_doc<T: PyClass>() -> PyResult<Option<*mut c_void>> {
    Ok(match T::DOC {
        "\0" => None,
        s if s.as_bytes().ends_with(b"\0") => Some(s.as_ptr() as _),
        // If the description is not null-terminated, create CString and leak it
        s => Some(CString::new(s)?.into_raw() as _),
    })
}

fn get_type_name<T: PyTypeInfo>(module_name: Option<&str>) -> PyResult<*mut c_char> {
    Ok(match module_name {
        Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME))?.into_raw(),
        None => CString::new(format!("builtins.{}", T::NAME))?.into_raw(),
    })
}

fn into_raw<T>(vec: Vec<T>) -> *mut c_void {
    Box::into_raw(vec.into_boxed_slice()) as _
}

pub(crate) fn create_type_object<T>(
    py: Python,
    module_name: Option<&str>,
) -> PyResult<*mut ffi::PyTypeObject>
where
    T: PyClass,
{
    let mut slots = TypeSlots::default();

    slots.push(ffi::Py_tp_base, T::BaseType::type_object_raw(py) as _);
    if let Some(doc) = tp_doc::<T>()? {
        slots.push(ffi::Py_tp_doc, doc);
    }

    slots.push(ffi::Py_tp_new, T::get_new().unwrap_or(fallback_new) as _);
    slots.push(ffi::Py_tp_dealloc, tp_dealloc::<T> as _);

    if let Some(alloc) = T::get_alloc() {
        slots.push(ffi::Py_tp_alloc, alloc as _);
    }
    if let Some(free) = T::get_free() {
        slots.push(ffi::Py_tp_free, free as _);
    }

    if let Some(call_meth) = T::get_call() {
        slots.push(ffi::Py_tp_call, call_meth as _);
    }

    if cfg!(Py_3_9) {
        let members = py_class_members::<T>();
        if !members.is_empty() {
            slots.push(ffi::Py_tp_members, into_raw(members))
        }
    }

    // normal methods
    let methods = py_class_method_defs(&T::for_each_method_def);
    if !methods.is_empty() {
        slots.push(ffi::Py_tp_methods, into_raw(methods));
    }

    // properties
    let props = py_class_properties(T::Dict::IS_DUMMY, &T::for_each_method_def);
    if !props.is_empty() {
        slots.push(ffi::Py_tp_getset, into_raw(props));
    }

    // protocol methods
    let mut has_gc_methods = false;
    T::for_each_proto_slot(&mut |proto_slots| {
        has_gc_methods |= proto_slots
            .iter()
            .any(|slot| slot.slot == ffi::Py_tp_clear || slot.slot == ffi::Py_tp_traverse);
        slots.0.extend_from_slice(proto_slots);
    });

    slots.push(0, ptr::null_mut());
    let mut spec = ffi::PyType_Spec {
        name: get_type_name::<T>(module_name)?,
        basicsize: std::mem::size_of::<T::Layout>() as c_int,
        itemsize: 0,
        flags: py_class_flags(has_gc_methods, T::IS_GC, T::IS_BASETYPE),
        slots: slots.0.as_mut_ptr(),
    };

    let type_object = unsafe { ffi::PyType_FromSpec(&mut spec) };
    if type_object.is_null() {
        Err(PyErr::api_call_failed(py))
    } else {
        tp_init_additional::<T>(type_object as _);
        Ok(type_object as _)
    }
}

/// Additional type initializations necessary before Python 3.10
#[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
fn tp_init_additional<T: PyClass>(type_object: *mut ffi::PyTypeObject) {
    // Just patch the type objects for the things there's no
    // PyType_FromSpec API for... there's no reason this should work,
    // except for that it does and we have tests.

    // Running this causes PyPy to segfault.
    #[cfg(all(not(PyPy), not(Py_3_10)))]
    {
        if T::DOC != "\0" {
            unsafe {
                // Until CPython 3.10, tp_doc was treated specially for
                // heap-types, and it removed the text_signature value from it.
                // We go in after the fact and replace tp_doc with something
                // that _does_ include the text_signature value!
                ffi::PyObject_Free((*type_object).tp_doc as _);
                let data = ffi::PyObject_Malloc(T::DOC.len());
                data.copy_from(T::DOC.as_ptr() as _, T::DOC.len());
                (*type_object).tp_doc = data as _;
            }
        }
    }

    // Setting buffer protocols via slots doesn't work until Python 3.9, so on older versions we
    // must manually fixup the type object.
    if cfg!(not(Py_3_9)) {
        if let Some(buffer) = T::get_buffer() {
            unsafe {
                (*(*type_object).tp_as_buffer).bf_getbuffer = buffer.bf_getbuffer;
                (*(*type_object).tp_as_buffer).bf_releasebuffer = buffer.bf_releasebuffer;
            }
        }
    }

    // Setting tp_dictoffset and tp_weaklistoffset via slots doesn't work until Python 3.9, so on
    // older versions again we must fixup the type object.
    if cfg!(not(Py_3_9)) {
        // __dict__ support
        if let Some(dict_offset) = PyCell::<T>::dict_offset() {
            unsafe {
                (*type_object).tp_dictoffset = dict_offset as ffi::Py_ssize_t;
            }
        }
        // weakref support
        if let Some(weakref_offset) = PyCell::<T>::weakref_offset() {
            unsafe {
                (*type_object).tp_weaklistoffset = weakref_offset as ffi::Py_ssize_t;
            }
        }
    }
}

#[cfg(any(Py_LIMITED_API, Py_3_10))]
fn tp_init_additional<T: PyClass>(_type_object: *mut ffi::PyTypeObject) {}

fn py_class_flags(has_gc_methods: bool, is_gc: bool, is_basetype: bool) -> c_uint {
    let mut flags = if has_gc_methods || is_gc {
        ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC
    } else {
        ffi::Py_TPFLAGS_DEFAULT
    };
    if is_basetype {
        flags |= ffi::Py_TPFLAGS_BASETYPE;
    }

    // `c_ulong` and `c_uint` have the same size
    // on some platforms (like windows)
    #[allow(clippy::useless_conversion)]
    flags.try_into().unwrap()
}

fn py_class_method_defs(
    for_each_method_def: &dyn Fn(&mut dyn FnMut(&[PyMethodDefType])),
) -> Vec<ffi::PyMethodDef> {
    let mut defs = Vec::new();

    for_each_method_def(&mut |method_defs| {
        defs.extend(method_defs.iter().filter_map(|def| match def {
            PyMethodDefType::Method(def)
            | PyMethodDefType::Class(def)
            | PyMethodDefType::Static(def) => Some(def.as_method_def().unwrap()),
            _ => None,
        }));
    });

    if !defs.is_empty() {
        defs.push(unsafe { std::mem::zeroed() });
    }

    defs
}

/// Generates the __dictoffset__ and __weaklistoffset__ members, to set tp_dictoffset and
/// tp_weaklistoffset.
///
/// Only works on Python 3.9 and up.
#[cfg(Py_3_9)]
fn py_class_members<T: PyClass>() -> Vec<ffi::structmember::PyMemberDef> {
    #[inline(always)]
    fn offset_def(name: &'static str, offset: usize) -> ffi::structmember::PyMemberDef {
        ffi::structmember::PyMemberDef {
            name: name.as_ptr() as _,
            type_code: ffi::structmember::T_PYSSIZET,
            offset: offset as _,
            flags: ffi::structmember::READONLY,
            doc: std::ptr::null_mut(),
        }
    }

    let mut members = Vec::new();

    // __dict__ support
    if let Some(dict_offset) = PyCell::<T>::dict_offset() {
        members.push(offset_def("__dictoffset__\0", dict_offset));
    }

    // weakref support
    if let Some(weakref_offset) = PyCell::<T>::weakref_offset() {
        members.push(offset_def("__weaklistoffset__\0", weakref_offset));
    }

    if !members.is_empty() {
        members.push(unsafe { std::mem::zeroed() });
    }

    members
}

// Stub needed since the `if cfg!()` above still compiles contained code.
#[cfg(not(Py_3_9))]
fn py_class_members<T: PyClass>() -> Vec<ffi::structmember::PyMemberDef> {
    vec![]
}

const PY_GET_SET_DEF_INIT: ffi::PyGetSetDef = ffi::PyGetSetDef {
    name: ptr::null_mut(),
    get: None,
    set: None,
    doc: ptr::null_mut(),
    closure: ptr::null_mut(),
};

#[allow(clippy::collapsible_if)] // for if cfg!
fn py_class_properties(
    is_dummy: bool,
    for_each_method_def: &dyn Fn(&mut dyn FnMut(&[PyMethodDefType])),
) -> Vec<ffi::PyGetSetDef> {
    let mut defs = std::collections::HashMap::new();

    for_each_method_def(&mut |method_defs| {
        for def in method_defs {
            match def {
                PyMethodDefType::Getter(getter) => {
                    getter.copy_to(defs.entry(getter.name).or_insert(PY_GET_SET_DEF_INIT));
                }
                PyMethodDefType::Setter(setter) => {
                    setter.copy_to(defs.entry(setter.name).or_insert(PY_GET_SET_DEF_INIT));
                }
                _ => (),
            }
        }
    });

    let mut props: Vec<_> = defs.values().cloned().collect();

    // PyPy doesn't automatically adds __dict__ getter / setter.
    // PyObject_GenericGetDict not in the limited API until Python 3.10.
    push_dict_getset(&mut props, is_dummy);

    if !props.is_empty() {
        props.push(unsafe { std::mem::zeroed() });
    }
    props
}

#[cfg(not(any(PyPy, all(Py_LIMITED_API, not(Py_3_10)))))]
fn push_dict_getset(props: &mut Vec<ffi::PyGetSetDef>, is_dummy: bool) {
    if !is_dummy {
        props.push(ffi::PyGetSetDef {
            name: "__dict__\0".as_ptr() as *mut c_char,
            get: Some(ffi::PyObject_GenericGetDict),
            set: Some(ffi::PyObject_GenericSetDict),
            doc: ptr::null_mut(),
            closure: ptr::null_mut(),
        });
    }
}

#[cfg(any(PyPy, all(Py_LIMITED_API, not(Py_3_10))))]
fn push_dict_getset(_: &mut Vec<ffi::PyGetSetDef>, _is_dummy: bool) {}
