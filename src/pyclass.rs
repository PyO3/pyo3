//! `PyClass` and related traits.
use crate::{
    class::impl_::{
        fallback_new, sq_ass_item_from_mapping, sq_item_from_mapping, sq_length_from_mapping,
        tp_dealloc, PyClassImpl,
    },
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

/// If `PyClass` is implemented for a Rust type `T`, then we can use `T` in the Python
/// world, via `PyCell`.
///
/// The `#[pyclass]` attribute automatically implements this trait for your Rust struct,
/// so you normally don't have to use this trait directly.
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

    #[cfg(Py_3_9)]
    {
        let members = py_class_members::<T>();
        if !members.is_empty() {
            slots.push(ffi::Py_tp_members, into_raw(members))
        }
    }

    let PyClassInfo {
        mut has_seqlen,
        mut has_getseqitem,
        mut has_setseqitem,
        method_defs,
        property_defs,
    } = method_defs_to_pyclass_info(&T::for_each_method_def, T::Dict::IS_DUMMY);

    // normal methods
    if !method_defs.is_empty() {
        slots.push(ffi::Py_tp_methods, into_raw(method_defs));
    }

    // properties
    if !property_defs.is_empty() {
        slots.push(ffi::Py_tp_getset, into_raw(property_defs));
    }

    // protocol methods
    let mut has_len = false;
    let mut has_getitem = false;
    let mut has_setitem = false;
    let mut has_gc_methods = false;
    T::for_each_proto_slot(&mut |proto_slots| {
        for slot in proto_slots {
            if !T::TRUE_MAPPING {
                has_len |= slot.slot == ffi::Py_mp_length;
                has_seqlen |= slot.slot == ffi::Py_sq_length;
                has_getitem |= slot.slot == ffi::Py_mp_subscript;
                has_getseqitem |= slot.slot == ffi::Py_sq_item;
                has_setitem |= slot.slot == ffi::Py_mp_ass_subscript;
                has_setseqitem |= slot.slot == ffi::Py_sq_ass_item;
            }
            has_gc_methods |= slot.slot == ffi::Py_tp_clear || slot.slot == ffi::Py_tp_traverse;
        }
        slots.0.extend_from_slice(proto_slots);
    });

    // If mapping methods implemented but not sequence methods, the sequence methods get some
    // default implementations. CPython does the same for Python `class` statements.

    if !T::TRUE_MAPPING {
        if has_len && !has_seqlen {
            slots.push(ffi::Py_sq_length, sq_length_from_mapping as _);
        }

        if has_getitem && !has_getseqitem {
            slots.push(ffi::Py_sq_item, sq_item_from_mapping as _);
        }

        if has_setitem && !has_setseqitem {
            slots.push(ffi::Py_sq_ass_item, sq_ass_item_from_mapping as _);
        }
    }

    // Add empty sentinel at the end
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
        Err(PyErr::fetch(py))
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
    #[cfg(not(Py_3_9))]
    {
        if let Some(buffer) = T::get_buffer() {
            unsafe {
                (*(*type_object).tp_as_buffer).bf_getbuffer = buffer.bf_getbuffer;
                (*(*type_object).tp_as_buffer).bf_releasebuffer = buffer.bf_releasebuffer;
            }
        }
    }

    // Setting tp_dictoffset and tp_weaklistoffset via slots doesn't work until Python 3.9, so on
    // older versions again we must fixup the type object.
    #[cfg(not(Py_3_9))]
    {
        // __dict__ support
        if let Some(dict_offset) = PyCell::<T>::dict_offset() {
            unsafe {
                (*type_object).tp_dictoffset = dict_offset;
            }
        }
        // weakref support
        if let Some(weakref_offset) = PyCell::<T>::weakref_offset() {
            unsafe {
                (*type_object).tp_weaklistoffset = weakref_offset;
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

struct PyClassInfo {
    has_seqlen: bool,
    has_getseqitem: bool,
    has_setseqitem: bool,
    method_defs: Vec<ffi::PyMethodDef>,
    property_defs: Vec<ffi::PyGetSetDef>,
}

fn method_defs_to_pyclass_info(
    for_each_method_def: &dyn Fn(&mut dyn FnMut(&[PyMethodDefType])),
    dict_is_dummy: bool,
) -> PyClassInfo {
    let mut has_seqlen = false;
    let mut has_getseqitem = false;
    let mut has_setseqitem = false;
    let mut method_defs = Vec::new();
    let mut property_defs_map = std::collections::HashMap::new();

    for_each_method_def(&mut |class_method_defs| {
        for def in class_method_defs {
            match def {
                PyMethodDefType::Getter(getter) => {
                    getter.copy_to(
                        property_defs_map
                            .entry(getter.name)
                            .or_insert(PY_GET_SET_DEF_INIT),
                    );
                }
                PyMethodDefType::Setter(setter) => {
                    setter.copy_to(
                        property_defs_map
                            .entry(setter.name)
                            .or_insert(PY_GET_SET_DEF_INIT),
                    );
                }
                PyMethodDefType::Method(def)
                | PyMethodDefType::Class(def)
                | PyMethodDefType::Static(def) => method_defs.push(def.as_method_def().unwrap()),
                PyMethodDefType::ClassAttribute(attr) => {
                    has_seqlen |= attr.name == "__seqlen__\0";
                    has_getseqitem |= attr.name == "__getseqitem__\0";
                    has_setseqitem |= attr.name == "__setseqitem__\0";
                }
            }
        }
    });

    // TODO: use into_values when on MSRV Rust >= 1.54
    let mut property_defs: Vec<_> = property_defs_map
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    if !method_defs.is_empty() {
        method_defs.push(unsafe { std::mem::zeroed() });
    }

    // PyPy doesn't automatically adds __dict__ getter / setter.
    // PyObject_GenericGetDict not in the limited API until Python 3.10.
    push_dict_getset(&mut property_defs, dict_is_dummy);

    if !property_defs.is_empty() {
        property_defs.push(unsafe { std::mem::zeroed() });
    }

    PyClassInfo {
        has_seqlen,
        has_getseqitem,
        has_setseqitem,
        method_defs,
        property_defs,
    }
}

/// Generates the __dictoffset__ and __weaklistoffset__ members, to set tp_dictoffset and
/// tp_weaklistoffset.
///
/// Only works on Python 3.9 and up.
#[cfg(Py_3_9)]
fn py_class_members<T: PyClass>() -> Vec<ffi::structmember::PyMemberDef> {
    #[inline(always)]
    fn offset_def(name: &'static str, offset: ffi::Py_ssize_t) -> ffi::structmember::PyMemberDef {
        ffi::structmember::PyMemberDef {
            name: name.as_ptr() as _,
            type_code: ffi::structmember::T_PYSSIZET,
            offset,
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

const PY_GET_SET_DEF_INIT: ffi::PyGetSetDef = ffi::PyGetSetDef {
    name: ptr::null_mut(),
    get: None,
    set: None,
    doc: ptr::null_mut(),
    closure: ptr::null_mut(),
};

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
