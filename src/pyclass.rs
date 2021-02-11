//! `PyClass` and related traits.
use crate::class::methods::{PyClassAttributeDef, PyMethodDefType, PyMethods};
use crate::class::proto_methods::PyProtoMethods;
use crate::derive_utils::PyBaseTypeUtils;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyLayout};
use crate::{ffi, PyCell, PyErr, PyNativeType, PyResult, PyTypeInfo, Python};
use std::convert::TryInto;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::{mem, ptr, thread};

#[inline]
unsafe fn get_type_alloc(tp: *mut ffi::PyTypeObject) -> Option<ffi::allocfunc> {
    mem::transmute(ffi::PyType_GetSlot(tp, ffi::Py_tp_alloc))
}

#[inline]
pub(crate) unsafe fn get_type_free(tp: *mut ffi::PyTypeObject) -> Option<ffi::freefunc> {
    mem::transmute(ffi::PyType_GetSlot(tp, ffi::Py_tp_free))
}

/// Workaround for Python issue 35810; no longer necessary in Python 3.8
#[inline]
#[cfg(not(Py_3_8))]
pub(crate) unsafe fn bpo_35810_workaround(_py: Python, ty: *mut ffi::PyTypeObject) {
    #[cfg(Py_LIMITED_API)]
    {
        // Must check version at runtime for abi3 wheels - they could run against a higher version
        // than the build config suggests.
        use crate::once_cell::GILOnceCell;
        static IS_PYTHON_3_8: GILOnceCell<bool> = GILOnceCell::new();

        if *IS_PYTHON_3_8.get_or_init(_py, || _py.version_info() >= (3, 8)) {
            // No fix needed - the wheel is running on a sufficiently new interpreter.
            return;
        }
    }

    ffi::Py_INCREF(ty as *mut ffi::PyObject);
}

#[inline]
pub(crate) unsafe fn default_new<T: PyTypeInfo>(
    py: Python,
    subtype: *mut ffi::PyTypeObject,
) -> *mut ffi::PyObject {
    // if the class derives native types(e.g., PyDict), call special new
    if T::FLAGS & type_flags::EXTENDED != 0 && T::BaseLayout::IS_NATIVE_TYPE {
        #[cfg(not(Py_LIMITED_API))]
        {
            let base_tp = T::BaseType::type_object_raw(py);
            if let Some(base_new) = (*base_tp).tp_new {
                return base_new(subtype, ptr::null_mut(), ptr::null_mut());
            }
        }
        #[cfg(Py_LIMITED_API)]
        {
            // Silence unused parameter warning.
            let _ = py;
            unreachable!("Subclassing native types isn't support in limited API mode");
        }
    }

    let alloc = get_type_alloc(subtype).unwrap_or(ffi::PyType_GenericAlloc);

    #[cfg(not(Py_3_8))]
    bpo_35810_workaround(py, subtype);

    alloc(subtype, 0)
}

/// This trait enables custom `tp_new`/`tp_dealloc` implementations for `T: PyClass`.
pub trait PyClassAlloc: PyTypeInfo + Sized {
    /// Allocate the actual field for `#[pyclass]`.
    ///
    /// # Safety
    /// This function must return a valid pointer to the Python heap.
    unsafe fn new(py: Python, subtype: *mut ffi::PyTypeObject) -> *mut Self::Layout {
        default_new::<Self>(py, subtype) as _
    }

    /// Deallocate `#[pyclass]` on the Python heap.
    ///
    /// # Safety
    /// `self_` must be a valid pointer to the Python heap.
    #[allow(clippy::clippy::collapsible_if)] // for if cfg!
    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);
        let obj = self_ as *mut ffi::PyObject;

        let ty = ffi::Py_TYPE(obj);
        let free = get_type_free(ty).unwrap_or_else(|| tp_free_fallback(ty));
        free(obj as *mut c_void);

        if cfg!(Py_3_8) {
            if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
                ffi::Py_DECREF(ty as *mut ffi::PyObject);
            }
        }
    }
}

unsafe extern "C" fn tp_dealloc<T>(obj: *mut ffi::PyObject)
where
    T: PyClassAlloc,
{
    let pool = crate::GILPool::new();
    let py = pool.python();
    <T as PyClassAlloc>::dealloc(py, (obj as *mut T::Layout) as _)
}

pub(crate) unsafe fn tp_free_fallback(ty: *mut ffi::PyTypeObject) -> ffi::freefunc {
    if ffi::PyType_IS_GC(ty) != 0 {
        ffi::PyObject_GC_Del
    } else {
        ffi::PyObject_Free
    }
}

/// If `PyClass` is implemented for `T`, then we can use `T` in the Python world,
/// via `PyCell`.
///
/// The `#[pyclass]` attribute automatically implements this trait for your Rust struct,
/// so you don't have to use this trait directly.
pub trait PyClass:
    PyTypeInfo<Layout = PyCell<Self>, AsRefTarget = PyCell<Self>>
    + Sized
    + PyClassSend
    + PyClassAlloc
    + PyMethods
    + PyProtoMethods
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
pub(crate) struct TypeSlots(Vec<ffi::PyType_Slot>);

impl TypeSlots {
    fn push(&mut self, slot: c_int, pfunc: *mut c_void) {
        self.0.push(ffi::PyType_Slot { slot, pfunc });
    }
}

fn tp_doc<T: PyClass>() -> PyResult<Option<*mut c_void>> {
    Ok(match T::DESCRIPTION {
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
    slots.push(ffi::Py_tp_dealloc, tp_dealloc::<T> as _);
    if let Some(doc) = tp_doc::<T>()? {
        slots.push(ffi::Py_tp_doc, doc);
    }

    let (new, call, methods) = py_class_method_defs::<T>();
    slots.push(ffi::Py_tp_new, new as _);
    if let Some(call_meth) = call {
        slots.push(ffi::Py_tp_call, call_meth as _);
    }

    if cfg!(Py_3_9) {
        let members = py_class_members::<T>();
        if !members.is_empty() {
            slots.push(ffi::Py_tp_members, into_raw(members))
        }
    }

    // normal methods
    if !methods.is_empty() {
        slots.push(ffi::Py_tp_methods, into_raw(methods));
    }

    // properties
    let props = py_class_properties::<T>();
    if !props.is_empty() {
        slots.push(ffi::Py_tp_getset, into_raw(props));
    }

    // protocol methods
    let mut has_gc_methods = false;
    T::for_each_proto_slot(|slot| {
        has_gc_methods |= slot.slot == ffi::Py_tp_clear;
        has_gc_methods |= slot.slot == ffi::Py_tp_traverse;
        slots.0.push(slot);
    });

    slots.push(0, ptr::null_mut());
    let mut spec = ffi::PyType_Spec {
        name: get_type_name::<T>(module_name)?,
        basicsize: std::mem::size_of::<T::Layout>() as c_int,
        itemsize: 0,
        flags: py_class_flags::<T>(has_gc_methods),
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
        if T::DESCRIPTION != "\0" {
            unsafe {
                // Until CPython 3.10, tp_doc was treated specially for
                // heap-types, and it removed the text_signature value from it.
                // We go in after the fact and replace tp_doc with something
                // that _does_ include the text_signature value!
                ffi::PyObject_Free((*type_object).tp_doc as _);
                let data = ffi::PyObject_Malloc(T::DESCRIPTION.len());
                data.copy_from(T::DESCRIPTION.as_ptr() as _, T::DESCRIPTION.len());
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

fn py_class_flags<T: PyClass + PyTypeInfo>(has_gc_methods: bool) -> c_uint {
    let mut flags = if has_gc_methods || T::FLAGS & type_flags::GC != 0 {
        ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC
    } else {
        ffi::Py_TPFLAGS_DEFAULT
    };
    if T::FLAGS & type_flags::BASETYPE != 0 {
        flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
    flags.try_into().unwrap()
}

pub(crate) fn py_class_attributes<T: PyMethods>() -> impl Iterator<Item = PyClassAttributeDef> {
    T::py_methods().into_iter().filter_map(|def| match def {
        PyMethodDefType::ClassAttribute(attr) => Some(attr.to_owned()),
        _ => None,
    })
}

unsafe extern "C" fn fallback_new(
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

fn py_class_method_defs<T: PyMethods>() -> (
    ffi::newfunc,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = fallback_new as ffi::newfunc;

    for def in T::py_methods() {
        match def {
            PyMethodDefType::New(def) => {
                new = def.ml_meth;
            }
            PyMethodDefType::Call(def) => {
                call = Some(def.ml_meth);
            }
            PyMethodDefType::Method(def)
            | PyMethodDefType::Class(def)
            | PyMethodDefType::Static(def) => {
                defs.push(def.as_method_def());
            }
            _ => (),
        }
    }

    if !defs.is_empty() {
        defs.push(ffi::PyMethodDef_INIT);
    }

    (new, call, defs)
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

#[allow(clippy::clippy::collapsible_if)] // for if cfg!
fn py_class_properties<T: PyClass>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = std::collections::HashMap::new();

    for def in T::py_methods() {
        match def {
            PyMethodDefType::Getter(getter) => {
                if !defs.contains_key(getter.name) {
                    #[allow(deprecated)]
                    let _ = defs.insert(getter.name.to_owned(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(getter.name).expect("Failed to call get_mut");
                getter.copy_to(def);
            }
            PyMethodDefType::Setter(setter) => {
                if !defs.contains_key(setter.name) {
                    #[allow(deprecated)]
                    let _ = defs.insert(setter.name.to_owned(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(setter.name).expect("Failed to call get_mut");
                setter.copy_to(def);
            }
            _ => (),
        }
    }

    let mut props: Vec<_> = defs.values().cloned().collect();

    // PyPy doesn't automatically adds __dict__ getter / setter.
    // PyObject_GenericGetDict not in the limited API until Python 3.10.
    push_dict_getset::<T>(&mut props);

    if !props.is_empty() {
        props.push(unsafe { std::mem::zeroed() });
    }
    props
}

#[cfg(not(any(PyPy, all(Py_LIMITED_API, not(Py_3_10)))))]
fn push_dict_getset<T: PyClass>(props: &mut Vec<ffi::PyGetSetDef>) {
    if !T::Dict::IS_DUMMY {
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
fn push_dict_getset<T: PyClass>(_: &mut Vec<ffi::PyGetSetDef>) {}

/// This trait is implemented for `#[pyclass]` and handles following two situations:
/// 1. In case `T` is `Send`, stub `ThreadChecker` is used and does nothing.
///    This implementation is used by default. Compile fails if `T: !Send`.
/// 2. In case `T` is `!Send`, `ThreadChecker` panics when `T` is accessed by another thread.
///    This implementation is used when `#[pyclass(unsendable)]` is given.
///    Panicking makes it safe to expose `T: !Send` to the Python interpreter, where all objects
///    can be accessed by multiple threads by `threading` module.
pub trait PyClassSend: Sized {
    type ThreadChecker: PyClassThreadChecker<Self>;
}

#[doc(hidden)]
pub trait PyClassThreadChecker<T>: Sized {
    fn ensure(&self);
    fn new() -> Self;
    private_decl! {}
}

/// Stub checker for `Send` types.
#[doc(hidden)]
pub struct ThreadCheckerStub<T: Send>(PhantomData<T>);

impl<T: Send> PyClassThreadChecker<T> for ThreadCheckerStub<T> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

impl<T: PyNativeType> PyClassThreadChecker<T> for ThreadCheckerStub<crate::PyObject> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

/// Thread checker for unsendable types.
/// Panics when the value is accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerImpl<T>(thread::ThreadId, PhantomData<T>);

impl<T> PyClassThreadChecker<T> for ThreadCheckerImpl<T> {
    fn ensure(&self) {
        if thread::current().id() != self.0 {
            panic!(
                "{} is unsendable, but sent to another thread!",
                std::any::type_name::<T>()
            );
        }
    }
    fn new() -> Self {
        ThreadCheckerImpl(thread::current().id(), PhantomData)
    }
    private_impl! {}
}

/// Thread checker for types that have `Send` and `extends=...`.
/// Ensures that `T: Send` and the parent is not accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerInherited<T: Send, U: PyBaseTypeUtils>(PhantomData<T>, U::ThreadChecker);

impl<T: Send, U: PyBaseTypeUtils> PyClassThreadChecker<T> for ThreadCheckerInherited<T, U> {
    fn ensure(&self) {
        self.1.ensure();
    }
    fn new() -> Self {
        ThreadCheckerInherited(PhantomData, U::ThreadChecker::new())
    }
    private_impl! {}
}
