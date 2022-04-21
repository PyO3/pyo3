//! `PyClass` and related traits.
use crate::pycell::{Immutable, Mutable};
use crate::{
    callback::IntoPyCallbackOutput,
    exceptions::PyTypeError,
    ffi,
    impl_::pyclass::{
        assign_sequence_item_from_mapping, get_sequence_item_from_mapping, tp_dealloc, PyClassImpl,
        PyClassItems,
    },
    IntoPy, IntoPyPointer, PyCell, PyErr, PyMethodDefType, PyObject, PyResult, PyTypeInfo, Python,
};
use std::{
    convert::TryInto,
    ffi::{CStr, CString},
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
}

pub trait MutablePyClass: PyClass<Mutability = Mutable> {}
pub trait ImmutablePyClass: PyClass<Mutability = Immutable> {}

impl<T> MutablePyClass for T where T: PyClass<Mutability = Mutable> {}
impl<T> ImmutablePyClass for T where T: PyClass<Mutability = Immutable> {}

fn into_raw<T>(vec: Vec<T>) -> *mut c_void {
    Box::into_raw(vec.into_boxed_slice()) as _
}

pub(crate) fn create_type_object<T>(py: Python<'_>) -> *mut ffi::PyTypeObject
where
    T: PyClass,
{
    match unsafe {
        create_type_object_impl(
            py,
            T::DOC,
            T::MODULE,
            T::NAME,
            T::BaseType::type_object_raw(py),
            std::mem::size_of::<T::Layout>(),
            tp_dealloc::<T>,
            T::dict_offset(),
            T::weaklist_offset(),
            &T::for_all_items,
            T::IS_BASETYPE,
            T::IS_MAPPING,
        )
    } {
        Ok(type_object) => type_object,
        Err(e) => type_object_creation_failed(py, e, T::NAME),
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_type_object_impl(
    py: Python<'_>,
    tp_doc: &str,
    module_name: Option<&str>,
    name: &str,
    base_type_object: *mut ffi::PyTypeObject,
    basicsize: usize,
    tp_dealloc: ffi::destructor,
    dict_offset: Option<ffi::Py_ssize_t>,
    weaklist_offset: Option<ffi::Py_ssize_t>,
    for_all_items: &dyn Fn(&mut dyn FnMut(&PyClassItems)),
    is_basetype: bool,
    is_mapping: bool,
) -> PyResult<*mut ffi::PyTypeObject> {
    let mut slots = Vec::new();

    fn push_slot(slots: &mut Vec<ffi::PyType_Slot>, slot: c_int, pfunc: *mut c_void) {
        slots.push(ffi::PyType_Slot { slot, pfunc });
    }

    push_slot(&mut slots, ffi::Py_tp_base, base_type_object as _);
    if let Some(doc) = py_class_doc(tp_doc) {
        push_slot(&mut slots, ffi::Py_tp_doc, doc as _);
    }

    push_slot(&mut slots, ffi::Py_tp_dealloc, tp_dealloc as _);

    #[cfg(Py_3_9)]
    {
        let members = py_class_members(dict_offset, weaklist_offset);
        if !members.is_empty() {
            push_slot(&mut slots, ffi::Py_tp_members, into_raw(members))
        }
    }

    let PyClassInfo {
        method_defs,
        property_defs,
    } = method_defs_to_pyclass_info(for_all_items, dict_offset.is_none());

    // normal methods
    if !method_defs.is_empty() {
        push_slot(&mut slots, ffi::Py_tp_methods, into_raw(method_defs));
    }

    // properties
    if !property_defs.is_empty() {
        push_slot(&mut slots, ffi::Py_tp_getset, into_raw(property_defs));
    }

    // protocol methods
    let mut has_new = false;
    let mut has_getitem = false;
    let mut has_setitem = false;
    let mut has_traverse = false;
    let mut has_clear = false;

    // Before Python 3.9, need to patch in buffer methods manually (they don't work in slots)
    #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
    let mut buffer_procs: ffi::PyBufferProcs = Default::default();

    for_all_items(&mut |items| {
        for slot in items.slots {
            match slot.slot {
                ffi::Py_tp_new => has_new = true,
                ffi::Py_mp_subscript => has_getitem = true,
                ffi::Py_mp_ass_subscript => has_setitem = true,
                ffi::Py_tp_traverse => has_traverse = true,
                ffi::Py_tp_clear => has_clear = true,
                #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
                ffi::Py_bf_getbuffer => {
                    // Safety: slot.pfunc is a valid function pointer
                    buffer_procs.bf_getbuffer = Some(std::mem::transmute(slot.pfunc));
                }
                #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
                ffi::Py_bf_releasebuffer => {
                    // Safety: slot.pfunc is a valid function pointer
                    buffer_procs.bf_releasebuffer = Some(std::mem::transmute(slot.pfunc));
                }
                _ => {}
            }
        }
        slots.extend_from_slice(items.slots);
    });

    if !is_mapping {
        // If mapping methods implemented, define sequence methods get implemented too.
        // CPython does the same for Python `class` statements.

        // NB we don't implement sq_length to avoid annoying CPython behaviour of automatically adding
        // the length to negative indices.

        // Don't add these methods for "pure" mappings.

        if has_getitem {
            push_slot(
                &mut slots,
                ffi::Py_sq_item,
                get_sequence_item_from_mapping as _,
            );
        }

        if has_setitem {
            push_slot(
                &mut slots,
                ffi::Py_sq_ass_item,
                assign_sequence_item_from_mapping as _,
            );
        }
    }

    if !has_new {
        push_slot(&mut slots, ffi::Py_tp_new, no_constructor_defined as _);
    }

    if has_clear && !has_traverse {
        return Err(PyTypeError::new_err(format!(
            "`#[pyclass]` {} implements __clear__ without __traverse__",
            name
        )));
    }

    // Add empty sentinel at the end
    push_slot(&mut slots, 0, ptr::null_mut());

    let mut spec = ffi::PyType_Spec {
        name: py_class_qualified_name(module_name, name)?,
        basicsize: basicsize as c_int,
        itemsize: 0,
        flags: py_class_flags(has_traverse, is_basetype),
        slots: slots.as_mut_ptr(),
    };

    let type_object = ffi::PyType_FromSpec(&mut spec);
    if type_object.is_null() {
        Err(PyErr::fetch(py))
    } else {
        tp_init_additional(
            type_object as _,
            tp_doc,
            #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
            &buffer_procs,
            #[cfg(not(Py_3_9))]
            dict_offset,
            #[cfg(not(Py_3_9))]
            weaklist_offset,
        );
        Ok(type_object as _)
    }
}

#[cold]
fn type_object_creation_failed(py: Python<'_>, e: PyErr, name: &'static str) -> ! {
    e.print(py);
    panic!("An error occurred while initializing class {}", name)
}

/// Additional type initializations necessary before Python 3.10
#[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
unsafe fn tp_init_additional(
    _type_object: *mut ffi::PyTypeObject,
    _tp_doc: &str,
    #[cfg(not(Py_3_9))] buffer_procs: &ffi::PyBufferProcs,
    #[cfg(not(Py_3_9))] dict_offset: Option<ffi::Py_ssize_t>,
    #[cfg(not(Py_3_9))] weaklist_offset: Option<ffi::Py_ssize_t>,
) {
    // Just patch the type objects for the things there's no
    // PyType_FromSpec API for... there's no reason this should work,
    // except for that it does and we have tests.

    // Running this causes PyPy to segfault.
    #[cfg(all(not(PyPy), not(Py_3_10)))]
    {
        if _tp_doc != "\0" {
            // Until CPython 3.10, tp_doc was treated specially for
            // heap-types, and it removed the text_signature value from it.
            // We go in after the fact and replace tp_doc with something
            // that _does_ include the text_signature value!
            ffi::PyObject_Free((*_type_object).tp_doc as _);
            let data = ffi::PyObject_Malloc(_tp_doc.len());
            data.copy_from(_tp_doc.as_ptr() as _, _tp_doc.len());
            (*_type_object).tp_doc = data as _;
        }
    }

    // Setting buffer protocols, tp_dictoffset and tp_weaklistoffset via slots doesn't work until
    // Python 3.9, so on older versions we must manually fixup the type object.
    #[cfg(not(Py_3_9))]
    {
        (*(*_type_object).tp_as_buffer).bf_getbuffer = buffer_procs.bf_getbuffer;
        (*(*_type_object).tp_as_buffer).bf_releasebuffer = buffer_procs.bf_releasebuffer;

        if let Some(dict_offset) = dict_offset {
            (*_type_object).tp_dictoffset = dict_offset;
        }

        if let Some(weaklist_offset) = weaklist_offset {
            (*_type_object).tp_weaklistoffset = weaklist_offset;
        }
    }
}

#[cfg(any(Py_LIMITED_API, Py_3_10))]
fn tp_init_additional(
    _type_object: *mut ffi::PyTypeObject,
    _tp_doc: &str,
    #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))] _buffer_procs: &ffi::PyBufferProcs,
    #[cfg(not(Py_3_9))] _dict_offset: Option<ffi::Py_ssize_t>,
    #[cfg(not(Py_3_9))] _weaklist_offset: Option<ffi::Py_ssize_t>,
) {
}

fn py_class_doc(class_doc: &str) -> Option<*mut c_char> {
    match class_doc {
        "\0" => None,
        s => {
            // To pass *mut pointer to python safely, leak a CString in whichever case
            let cstring = if s.as_bytes().last() == Some(&0) {
                CStr::from_bytes_with_nul(s.as_bytes())
                    .unwrap_or_else(|e| panic!("doc contains interior nul byte: {:?} in {}", e, s))
                    .to_owned()
            } else {
                CString::new(s)
                    .unwrap_or_else(|e| panic!("doc contains interior nul byte: {:?} in {}", e, s))
            };
            Some(cstring.into_raw())
        }
    }
}

fn py_class_qualified_name(module_name: Option<&str>, class_name: &str) -> PyResult<*mut c_char> {
    Ok(CString::new(format!(
        "{}.{}",
        module_name.unwrap_or("builtins"),
        class_name
    ))?
    .into_raw())
}

fn py_class_flags(is_gc: bool, is_basetype: bool) -> c_uint {
    let mut flags = ffi::Py_TPFLAGS_DEFAULT;

    if is_gc {
        flags |= ffi::Py_TPFLAGS_HAVE_GC;
    }

    if is_basetype {
        flags |= ffi::Py_TPFLAGS_BASETYPE;
    }

    // `c_ulong` and `c_uint` have the same size
    // on some platforms (like windows)
    #[allow(clippy::useless_conversion)]
    flags.try_into().unwrap()
}

struct PyClassInfo {
    method_defs: Vec<ffi::PyMethodDef>,
    property_defs: Vec<ffi::PyGetSetDef>,
}

fn method_defs_to_pyclass_info(
    for_all_items: &dyn Fn(&mut dyn FnMut(&PyClassItems)),
    has_dict: bool,
) -> PyClassInfo {
    let mut method_defs = Vec::new();
    let mut property_defs_map = std::collections::HashMap::new();

    for_all_items(&mut |items| {
        for def in items.methods {
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
                PyMethodDefType::ClassAttribute(_) => {}
            }
        }
    });

    // TODO: use into_values when on MSRV Rust >= 1.54
    let mut property_defs: Vec<_> = property_defs_map
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    if !method_defs.is_empty() {
        // Safety: Python expects a zeroed entry to mark the end of the defs
        method_defs.push(unsafe { std::mem::zeroed() });
    }

    // PyPy doesn't automatically add __dict__ getter / setter.
    // PyObject_GenericGetDict not in the limited API until Python 3.10.
    if !has_dict {
        #[cfg(not(any(PyPy, all(Py_LIMITED_API, not(Py_3_10)))))]
        property_defs.push(ffi::PyGetSetDef {
            name: "__dict__\0".as_ptr() as *mut c_char,
            get: Some(ffi::PyObject_GenericGetDict),
            set: Some(ffi::PyObject_GenericSetDict),
            doc: ptr::null_mut(),
            closure: ptr::null_mut(),
        });
    }

    if !property_defs.is_empty() {
        // Safety: Python expects a zeroed entry to mark the end of the defs
        property_defs.push(unsafe { std::mem::zeroed() });
    }

    PyClassInfo {
        method_defs,
        property_defs,
    }
}

/// Generates the __dictoffset__ and __weaklistoffset__ members, to set tp_dictoffset and
/// tp_weaklistoffset.
///
/// Only works on Python 3.9 and up.
#[cfg(Py_3_9)]
fn py_class_members(
    dict_offset: Option<isize>,
    weaklist_offset: Option<isize>,
) -> Vec<ffi::structmember::PyMemberDef> {
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
    if let Some(dict_offset) = dict_offset {
        members.push(offset_def("__dictoffset__\0", dict_offset));
    }

    // weakref support
    if let Some(weaklist_offset) = weaklist_offset {
        members.push(offset_def("__weaklistoffset__\0", weaklist_offset));
    }

    if !members.is_empty() {
        // Safety: Python expects a zeroed entry to mark the end of the defs
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

/// Operators for the `__richcmp__` method
#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    /// The *less than* operator.
    Lt = ffi::Py_LT as isize,
    /// The *less than or equal to* operator.
    Le = ffi::Py_LE as isize,
    /// The equality operator.
    Eq = ffi::Py_EQ as isize,
    /// The *not equal to* operator.
    Ne = ffi::Py_NE as isize,
    /// The *greater than* operator.
    Gt = ffi::Py_GT as isize,
    /// The *greater than or equal to* operator.
    Ge = ffi::Py_GE as isize,
}

impl CompareOp {
    pub fn from_raw(op: c_int) -> Option<Self> {
        match op {
            ffi::Py_LT => Some(CompareOp::Lt),
            ffi::Py_LE => Some(CompareOp::Le),
            ffi::Py_EQ => Some(CompareOp::Eq),
            ffi::Py_NE => Some(CompareOp::Ne),
            ffi::Py_GT => Some(CompareOp::Gt),
            ffi::Py_GE => Some(CompareOp::Ge),
            _ => None,
        }
    }
}

/// Output of `__next__` which can either `yield` the next value in the iteration, or
/// `return` a value to raise `StopIteration` in Python.
///
/// See [`PyIterProtocol`](trait.PyIterProtocol.html) for an example.
pub enum IterNextOutput<T, U> {
    /// The value yielded by the iterator.
    Yield(T),
    /// The `StopIteration` object.
    Return(U),
}

pub type PyIterNextOutput = IterNextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterNextOutput {
    fn convert(self, _py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterNextOutput::Yield(o) => Ok(o.into_ptr()),
            IterNextOutput::Return(opt) => Err(crate::exceptions::PyStopIteration::new_err((opt,))),
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterNextOutput> for IterNextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterNextOutput> {
        match self {
            IterNextOutput::Yield(o) => Ok(IterNextOutput::Yield(o.into_py(py))),
            IterNextOutput::Return(o) => Ok(IterNextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterNextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterNextOutput> {
        match self {
            Some(o) => Ok(PyIterNextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterNextOutput::Return(py.None())),
        }
    }
}

/// Output of `__anext__`.
///
/// <https://docs.python.org/3/reference/expressions.html#agen.__anext__>
pub enum IterANextOutput<T, U> {
    /// An expression which the generator yielded.
    Yield(T),
    /// A `StopAsyncIteration` object.
    Return(U),
}

/// An [IterANextOutput] of Python objects.
pub type PyIterANextOutput = IterANextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterANextOutput {
    fn convert(self, _py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterANextOutput::Yield(o) => Ok(o.into_ptr()),
            IterANextOutput::Return(opt) => {
                Err(crate::exceptions::PyStopAsyncIteration::new_err((opt,)))
            }
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterANextOutput> for IterANextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterANextOutput> {
        match self {
            IterANextOutput::Yield(o) => Ok(IterANextOutput::Yield(o.into_py(py))),
            IterANextOutput::Return(o) => Ok(IterANextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterANextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterANextOutput> {
        match self {
            Some(o) => Ok(PyIterANextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterANextOutput::Return(py.None())),
        }
    }
}

/// Default new implementation
pub(crate) unsafe extern "C" fn no_constructor_defined(
    _subtype: *mut ffi::PyTypeObject,
    _args: *mut ffi::PyObject,
    _kwds: *mut ffi::PyObject,
) -> *mut ffi::PyObject {
    crate::callback_body!(py, {
        Err::<(), _>(crate::exceptions::PyTypeError::new_err(
            "No constructor defined",
        ))
    })
}
