//! `PyClass` and related traits.
use crate::class::methods::{PyClassAttributeDef, PyMethodDefType, PyMethods};
use crate::class::proto_methods::PyProtoMethods;
use crate::conversion::{AsPyPointer, FromPyPointer};
use crate::derive_utils::PyBaseTypeUtils;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyLayout};
use crate::types::PyAny;
use crate::{class, ffi, PyCell, PyErr, PyNativeType, PyResult, PyTypeInfo, Python};
use std::convert::TryInto;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint, c_void};
use std::{ptr, thread};

#[inline]
pub(crate) unsafe fn default_new<T: PyTypeInfo>(
    py: Python,
    subtype: *mut ffi::PyTypeObject,
) -> *mut ffi::PyObject {
    // if the class derives native types(e.g., PyDict), call special new
    if T::FLAGS & type_flags::EXTENDED != 0 && T::BaseLayout::IS_NATIVE_TYPE {
        let base_tp = T::BaseType::type_object_raw(py);
        if let Some(base_new) = (*base_tp).tp_new {
            return base_new(subtype, ptr::null_mut(), ptr::null_mut());
        }
    }
    let alloc = (*subtype).tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
    alloc(subtype, 0) as _
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
    unsafe fn dealloc(py: Python, self_: *mut Self::Layout) {
        (*self_).py_drop(py);
        let obj = PyAny::from_borrowed_ptr_or_panic(py, self_ as _);
        if Self::is_exact_instance(obj) && ffi::PyObject_CallFinalizerFromDealloc(obj.as_ptr()) < 0
        {
            // tp_finalize resurrected.
            return;
        }

        match (*ffi::Py_TYPE(obj.as_ptr())).tp_free {
            Some(free) => free(obj.as_ptr() as *mut c_void),
            None => tp_free_fallback(obj.as_ptr()),
        }
    }
}

fn tp_dealloc<T: PyClassAlloc>() -> Option<ffi::destructor> {
    unsafe extern "C" fn dealloc<T>(obj: *mut ffi::PyObject)
    where
        T: PyClassAlloc,
    {
        let pool = crate::GILPool::new();
        let py = pool.python();
        <T as PyClassAlloc>::dealloc(py, (obj as *mut T::Layout) as _)
    }
    Some(dealloc::<T>)
}

pub(crate) unsafe fn tp_free_fallback(obj: *mut ffi::PyObject) {
    let ty = ffi::Py_TYPE(obj);
    if ffi::PyType_IS_GC(ty) != 0 {
        ffi::PyObject_GC_Del(obj as *mut c_void);
    } else {
        ffi::PyObject_Free(obj as *mut c_void);
    }

    // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
    // so we need to call DECREF here:
    if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
        ffi::Py_DECREF(ty as *mut ffi::PyObject);
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

pub(crate) fn maybe_push_slot(
    slots: &mut Vec<ffi::PyType_Slot>,
    slot: c_int,
    val: Option<*mut c_void>,
) {
    if let Some(v) = val {
        slots.push(ffi::PyType_Slot { slot, pfunc: v });
    }
}

fn push_numbers_slots(slots: &mut Vec<ffi::PyType_Slot>, numbers: &ffi::PyNumberMethods) {
    maybe_push_slot(
        slots,
        ffi::Py_nb_add,
        numbers.nb_add.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_subtract,
        numbers.nb_subtract.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_multiply,
        numbers.nb_multiply.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_remainder,
        numbers.nb_remainder.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_divmod,
        numbers.nb_divmod.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_power,
        numbers.nb_power.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_negative,
        numbers.nb_negative.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_positive,
        numbers.nb_positive.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_absolute,
        numbers.nb_absolute.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_bool,
        numbers.nb_bool.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_invert,
        numbers.nb_invert.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_lshift,
        numbers.nb_lshift.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_rshift,
        numbers.nb_rshift.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_and,
        numbers.nb_and.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_xor,
        numbers.nb_xor.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_or,
        numbers.nb_or.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_int,
        numbers.nb_int.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_float,
        numbers.nb_float.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_add,
        numbers.nb_inplace_add.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_subtract,
        numbers.nb_inplace_subtract.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_multiply,
        numbers.nb_inplace_multiply.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_remainder,
        numbers.nb_inplace_remainder.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_power,
        numbers.nb_inplace_power.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_lshift,
        numbers.nb_inplace_lshift.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_rshift,
        numbers.nb_inplace_rshift.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_and,
        numbers.nb_inplace_and.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_xor,
        numbers.nb_inplace_xor.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_or,
        numbers.nb_inplace_or.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_floor_divide,
        numbers.nb_floor_divide.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_true_divide,
        numbers.nb_true_divide.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_floor_divide,
        numbers.nb_inplace_floor_divide.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_true_divide,
        numbers.nb_inplace_true_divide.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_index,
        numbers.nb_index.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_matrix_multiply,
        numbers.nb_matrix_multiply.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_nb_inplace_matrix_multiply,
        numbers.nb_inplace_matrix_multiply.map(|v| v as *mut c_void),
    );
}

fn push_mapping_slots(slots: &mut Vec<ffi::PyType_Slot>, mapping: &ffi::PyMappingMethods) {
    maybe_push_slot(
        slots,
        ffi::Py_mp_length,
        mapping.mp_length.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_mp_subscript,
        mapping.mp_subscript.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_mp_ass_subscript,
        mapping.mp_ass_subscript.map(|v| v as *mut c_void),
    );
}

fn push_sequence_slots(slots: &mut Vec<ffi::PyType_Slot>, seq: &ffi::PySequenceMethods) {
    maybe_push_slot(
        slots,
        ffi::Py_sq_length,
        seq.sq_length.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_concat,
        seq.sq_concat.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_repeat,
        seq.sq_repeat.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_item,
        seq.sq_item.map(|v| v as *mut c_void),
    );

    maybe_push_slot(
        slots,
        ffi::Py_sq_ass_item,
        seq.sq_ass_item.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_contains,
        seq.sq_contains.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_inplace_concat,
        seq.sq_inplace_concat.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_sq_inplace_repeat,
        seq.sq_inplace_repeat.map(|v| v as *mut c_void),
    );
}

fn push_async_slots(slots: &mut Vec<ffi::PyType_Slot>, asnc: &ffi::PyAsyncMethods) {
    maybe_push_slot(
        slots,
        ffi::Py_am_await,
        asnc.am_await.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_am_aiter,
        asnc.am_aiter.map(|v| v as *mut c_void),
    );
    maybe_push_slot(
        slots,
        ffi::Py_am_anext,
        asnc.am_anext.map(|v| v as *mut c_void),
    );
}

pub(crate) fn create_type_object<T>(
    py: Python,
    module_name: Option<&str>,
) -> PyResult<*mut ffi::PyTypeObject>
where
    T: PyClass,
{
    let mut slots = vec![];

    slots.push(ffi::PyType_Slot {
        slot: ffi::Py_tp_base,
        pfunc: T::BaseType::type_object_raw(py) as *mut c_void,
    });

    let doc = match T::DESCRIPTION {
        "\0" => None,
        s if s.as_bytes().ends_with(b"\0") => Some(s.as_ptr() as _),
        // If the description is not null-terminated, create CString and leak it
        s => Some(CString::new(s)?.into_raw() as _),
    };
    maybe_push_slot(&mut slots, ffi::Py_tp_doc, doc);

    maybe_push_slot(
        &mut slots,
        ffi::Py_tp_dealloc,
        tp_dealloc::<T>().map(|v| v as *mut c_void),
    );

    let (new, call, mut methods) = py_class_method_defs::<T>();
    maybe_push_slot(&mut slots, ffi::Py_tp_new, new.map(|v| v as *mut c_void));
    maybe_push_slot(&mut slots, ffi::Py_tp_call, call.map(|v| v as *mut c_void));
    // normal methods
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        maybe_push_slot(
            &mut slots,
            ffi::Py_tp_methods,
            Some(Box::into_raw(methods.into_boxed_slice()) as *mut c_void),
        );
    }

    // properties
    let mut props = py_class_properties::<T>();

    if !T::Dict::IS_DUMMY {
        props.push(ffi::PyGetSetDef_DICT);
    }
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        maybe_push_slot(
            &mut slots,
            ffi::Py_tp_getset,
            Some(Box::into_raw(props.into_boxed_slice()) as *mut c_void),
        );
    }

    if let Some(basic) = T::basic_methods() {
        unsafe { basic.as_ref() }.update_slots(&mut slots);
    }

    if let Some(number) = T::number_methods() {
        push_numbers_slots(&mut slots, unsafe { number.as_ref() });
    }

    // iterator methods
    if let Some(iter) = T::iter_methods() {
        unsafe { iter.as_ref() }.update_slots(&mut slots);
    }

    // mapping methods
    if let Some(mapping) = T::mapping_methods() {
        push_mapping_slots(&mut slots, unsafe { mapping.as_ref() });
    }

    // sequence methods
    if let Some(seq) = T::sequence_methods() {
        push_sequence_slots(&mut slots, unsafe { seq.as_ref() });
    }

    // descriptor protocol
    if let Some(descr) = T::descr_methods() {
        unsafe { descr.as_ref() }.update_slots(&mut slots);
    }

    // async methods
    if let Some(asnc) = T::async_methods() {
        push_async_slots(&mut slots, unsafe { asnc.as_ref() });
    }

    // GC support
    if let Some(gc) = T::gc_methods() {
        unsafe { gc.as_ref() }.update_slots(&mut slots);
    }

    slots.push(ffi::PyType_Slot {
        slot: 0,
        pfunc: ptr::null_mut(),
    });
    let mut spec = ffi::PyType_Spec {
        name: match module_name {
            Some(module_name) => CString::new(format!("{}.{}", module_name, T::NAME))?.into_raw(),
            None => CString::new(T::NAME)?.into_raw(),
        },
        basicsize: std::mem::size_of::<T::Layout>() as c_int,
        itemsize: 0,
        flags: py_class_flags::<T>(),
        slots: slots.as_mut_slice().as_mut_ptr(),
    };

    let type_object = unsafe { ffi::PyType_FromSpec(&mut spec) };
    if type_object.is_null() {
        PyErr::fetch(py).into()
    } else {
        // Just patch the type objects for the things there's no
        // PyType_FromSpec API for... there's no reason this should work,
        // except for that it does and we have tests.
        let mut type_object = type_object as *mut ffi::PyTypeObject;
        if let Some(buffer) = T::buffer_methods() {
            unsafe {
                (*(*type_object).tp_as_buffer).bf_getbuffer = buffer.as_ref().bf_getbuffer;
                (*(*type_object).tp_as_buffer).bf_releasebuffer = buffer.as_ref().bf_releasebuffer;
            }
        }
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
        Ok(type_object)
    }
}

fn py_class_flags<T: PyClass + PyTypeInfo>() -> c_uint {
    let mut flags = if T::gc_methods().is_some() || T::FLAGS & type_flags::GC != 0 {
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
        PyMethodDefType::ClassAttribute(attr) => Some(*attr),
        _ => None,
    })
}

fn fallback_new() -> Option<ffi::newfunc> {
    unsafe extern "C" fn fallback_new(
        _subtype: *mut ffi::PyTypeObject,
        _args: *mut ffi::PyObject,
        _kwds: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject {
        crate::callback_body!(py, {
            Err::<(), _>(crate::exceptions::PyTypeError::py_err(
                "No constructor defined",
            ))
        })
    }
    Some(fallback_new)
}

fn py_class_method_defs<T: PyMethods>() -> (
    Option<ffi::newfunc>,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = fallback_new();

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::New(ref def) => {
                if let class::methods::PyMethodType::PyNewFunc(meth) = def.ml_meth {
                    new = Some(meth)
                }
            }
            PyMethodDefType::Call(ref def) => {
                if let class::methods::PyMethodType::PyCFunctionWithKeywords(meth) = def.ml_meth {
                    call = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_call slot")
                }
            }
            PyMethodDefType::Method(ref def)
            | PyMethodDefType::Class(ref def)
            | PyMethodDefType::Static(ref def) => {
                defs.push(def.as_method_def());
            }
            _ => (),
        }
    }

    (new, call, defs)
}

fn py_class_properties<T: PyMethods>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = std::collections::HashMap::new();

    for def in T::py_methods() {
        match *def {
            PyMethodDefType::Getter(ref getter) => {
                let name = getter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                getter.copy_to(def);
            }
            PyMethodDefType::Setter(ref setter) => {
                let name = setter.name.to_string();
                if !defs.contains_key(&name) {
                    let _ = defs.insert(name.clone(), ffi::PyGetSetDef_INIT);
                }
                let def = defs.get_mut(&name).expect("Failed to call get_mut");
                setter.copy_to(def);
            }
            _ => (),
        }
    }

    defs.values().cloned().collect()
}

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
