//! An experiment module which has all codes related only to #[pyclass]
use crate::class::methods::{PyMethodDefType, PyMethodsProtocol};
use crate::conversion::{AsPyPointer, FromPyPointer, ToPyObject};
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{type_flags, PyConcreteObject, PyTypeObject};
use crate::{class, ffi, gil, PyErr, PyObject, PyResult, PyTypeInfo, Python};
use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::os::raw::c_void;
use std::ptr::{self, NonNull};

/// A trait that enables custome alloc/dealloc implementations for pyclasses.
pub trait PyClassAlloc: PyTypeInfo + Sized {
    unsafe fn alloc(_py: Python) -> *mut Self::ConcreteLayout {
        let tp_ptr = Self::type_object();
        let alloc = (*tp_ptr).tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
        alloc(tp_ptr, 0) as _
    }

    unsafe fn dealloc(py: Python, self_: *mut Self::ConcreteLayout) {
        (*self_).py_drop(py);
        let obj = self_ as _;
        if ffi::PyObject_CallFinalizerFromDealloc(obj) < 0 {
            return;
        }

        match Self::type_object().tp_free {
            Some(free) => free(obj as *mut c_void),
            None => tp_free_fallback(obj),
        }
    }
}

#[doc(hidden)]
pub unsafe fn tp_free_fallback(obj: *mut ffi::PyObject) {
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

pub trait PyClass: PyTypeInfo + Sized + PyClassAlloc + PyMethodsProtocol {
    type Dict: PyClassDict;
    type WeakRef: PyClassWeakRef;
}

unsafe impl<T> PyTypeObject for T
where
    T: PyClass,
{
    fn init_type() -> NonNull<ffi::PyTypeObject> {
        let type_object = unsafe { <Self as PyTypeInfo>::type_object() };

        if (type_object.tp_flags & ffi::Py_TPFLAGS_READY) == 0 {
            // automatically initialize the class on-demand
            let gil = Python::acquire_gil();
            let py = gil.python();

            initialize_type::<Self>(py, <Self as PyTypeInfo>::MODULE).unwrap_or_else(|e| {
                e.print(py);
                panic!("An error occurred while initializing class {}", Self::NAME)
            });
        }

        unsafe { NonNull::new_unchecked(type_object) }
    }
}

/// So this is a shell for our *sweet* pyclasses to survive in *harsh* Python world.
#[repr(C)]
pub struct PyClassShell<T: PyClass> {
    ob_base: <T::BaseType as PyTypeInfo>::ConcreteLayout,
    pyclass: ManuallyDrop<T>,
    dict: T::Dict,
    weakref: T::WeakRef,
}

impl<T: PyClass> PyClassShell<T> {
    pub fn new_ref(py: Python, value: T) -> PyResult<&Self> {
        unsafe {
            let ptr = Self::new(py, value)?;
            FromPyPointer::from_owned_ptr_or_err(py, ptr as _)
        }
    }

    pub fn new_mut(py: Python, value: T) -> PyResult<&mut Self> {
        unsafe {
            let ptr = Self::new(py, value)?;
            FromPyPointer::from_owned_ptr_or_err(py, ptr as _)
        }
    }

    pub unsafe fn new(py: Python, value: T) -> PyResult<*mut Self> {
        T::init_type();
        let base = T::alloc(py);
        if base.is_null() {
            return Err(PyErr::fetch(py));
        }
        let self_ = base as *mut Self;
        (*self_).pyclass = ManuallyDrop::new(value);
        (*self_).dict = T::Dict::new();
        (*self_).weakref = T::WeakRef::new();
        Ok(self_)
    }
}

impl<T: PyClass> PyConcreteObject for PyClassShell<T> {
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.pyclass);
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(self.as_ptr(), py);
        self.ob_base.py_drop(py);
    }
}

impl<T: PyClass> std::ops::Deref for PyClassShell<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.pyclass.deref()
    }
}

impl<T: PyClass> std::ops::DerefMut for PyClassShell<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.pyclass.deref_mut()
    }
}

impl<T: PyClass> ToPyObject for PyClassShell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for &'p PyClassShell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &**(gil::register_owned(py, p) as *const _ as *const Self))
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &**(gil::register_borrowed(py, p) as *const _ as *const Self))
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for &'p mut PyClassShell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &mut **(gil::register_owned(py, p) as *const _ as *mut Self))
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|p| &mut **(gil::register_borrowed(py, p) as *const _ as *mut Self))
    }
}

/// Register new type in python object system.
#[cfg(not(Py_LIMITED_API))]
pub fn initialize_type<T>(py: Python, module_name: Option<&str>) -> PyResult<*mut ffi::PyTypeObject>
where
    T: PyClass,
{
    let type_object: &mut ffi::PyTypeObject = unsafe { T::type_object() };
    let base_type_object: &mut ffi::PyTypeObject =
        unsafe { <T::BaseType as PyTypeInfo>::type_object() };

    // PyPy will segfault if passed only a nul terminator as `tp_doc`.
    // ptr::null() is OK though.
    if T::DESCRIPTION == "\0" {
        type_object.tp_doc = ptr::null();
    } else {
        type_object.tp_doc = T::DESCRIPTION.as_ptr() as *const _;
    };

    type_object.tp_base = base_type_object;

    let name = match module_name {
        Some(module_name) => format!("{}.{}", module_name, T::NAME),
        None => T::NAME.to_string(),
    };
    let name = CString::new(name).expect("Module name/type name must not contain NUL byte");
    type_object.tp_name = name.into_raw();

    // dealloc
    unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where
        T: PyClassAlloc,
    {
        let py = Python::assume_gil_acquired();
        let _pool = gil::GILPool::new_no_pointers(py);
        <T as PyClassAlloc>::dealloc(py, (obj as *mut T::ConcreteLayout) as _)
    }
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // type size
    type_object.tp_basicsize = std::mem::size_of::<T::ConcreteLayout>() as ffi::Py_ssize_t;

    let mut offset = type_object.tp_basicsize;

    // __dict__ support
    if let Some(dict_offset) = T::Dict::OFFSET {
        offset += dict_offset as ffi::Py_ssize_t;
        type_object.tp_dictoffset = offset;
    }

    // weakref support
    if let Some(weakref_offset) = T::WeakRef::OFFSET {
        offset += weakref_offset as ffi::Py_ssize_t;
        type_object.tp_weaklistoffset = offset;
    }

    // GC support
    <T as class::gc::PyGCProtocolImpl>::update_type_object(type_object);

    // descriptor protocol
    <T as class::descr::PyDescrProtocolImpl>::tp_as_descr(type_object);

    // iterator methods
    <T as class::iter::PyIterProtocolImpl>::tp_as_iter(type_object);

    // basic methods
    <T as class::basic::PyObjectProtocolImpl>::tp_as_object(type_object);

    fn to_ptr<T>(value: Option<T>) -> *mut T {
        value
            .map(|v| Box::into_raw(Box::new(v)))
            .unwrap_or_else(ptr::null_mut)
    }

    // number methods
    type_object.tp_as_number = to_ptr(<T as class::number::PyNumberProtocolImpl>::tp_as_number());
    // mapping methods
    type_object.tp_as_mapping =
        to_ptr(<T as class::mapping::PyMappingProtocolImpl>::tp_as_mapping());
    // sequence methods
    type_object.tp_as_sequence =
        to_ptr(<T as class::sequence::PySequenceProtocolImpl>::tp_as_sequence());
    // async methods
    type_object.tp_as_async = to_ptr(<T as class::pyasync::PyAsyncProtocolImpl>::tp_as_async());
    // buffer protocol
    type_object.tp_as_buffer = to_ptr(<T as class::buffer::PyBufferProtocolImpl>::tp_as_buffer());

    // normal methods
    let (new, call, mut methods) = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = Box::into_raw(methods.into_boxed_slice()) as *mut _;
    }

    // __new__ method
    type_object.tp_new = new;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();

    if T::Dict::OFFSET.is_some() {
        props.push(ffi::PyGetSetDef_DICT);
    }
    if !props.is_empty() {
        props.push(ffi::PyGetSetDef_INIT);
        type_object.tp_getset = Box::into_raw(props.into_boxed_slice()) as *mut _;
    }

    // set type flags
    py_class_flags::<T>(type_object);

    // register type object
    unsafe {
        if ffi::PyType_Ready(type_object) == 0 {
            Ok(type_object as *mut ffi::PyTypeObject)
        } else {
            PyErr::fetch(py).into()
        }
    }
}

fn py_class_flags<T: PyTypeInfo>(type_object: &mut ffi::PyTypeObject) {
    if type_object.tp_traverse != None
        || type_object.tp_clear != None
        || T::FLAGS & type_flags::GC != 0
    {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC;
    } else {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT;
    }
    if T::FLAGS & type_flags::BASETYPE != 0 {
        type_object.tp_flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
}

fn py_class_method_defs<T: PyMethodsProtocol>() -> (
    Option<ffi::newfunc>,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = None;

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

    for def in <T as class::basic::PyObjectProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::context::PyContextProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::mapping::PyMappingProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::number::PyNumberProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
    for def in <T as class::descr::PyDescrProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }

    py_class_async_methods::<T>(&mut defs);

    (new, call, defs)
}

fn py_class_async_methods<T>(defs: &mut Vec<ffi::PyMethodDef>) {
    for def in <T as class::pyasync::PyAsyncProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
}

fn py_class_properties<T: PyMethodsProtocol>() -> Vec<ffi::PyGetSetDef> {
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
