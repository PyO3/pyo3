// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python type object information

use crate::class::methods::PyMethodDefType;
use crate::err::{PyErr, PyResult};
use crate::instance::{Py, PyNativeType};
use crate::types::PyAny;
use crate::types::PyType;
use crate::AsPyPointer;
use crate::IntoPyPointer;
use crate::Python;
use crate::{class, ffi, gil};
use class::methods::PyMethodsProtocol;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::{self, NonNull};

/// Python type information.
pub trait PyTypeInfo {
    /// Type of objects to store in PyObject struct
    type Type;

    /// Class name
    const NAME: &'static str;

    /// Module name, if any
    const MODULE: Option<&'static str>;

    /// Class doc string
    const DESCRIPTION: &'static str = "\0";

    /// Size of the rust PyObject structure (PyObject + rust structure)
    const SIZE: usize;

    /// `Type` instance offset inside PyObject structure
    const OFFSET: isize;

    /// Type flags (ie PY_TYPE_FLAG_GC, PY_TYPE_FLAG_WEAKREF)
    const FLAGS: usize = 0;

    /// Base class
    type BaseType: PyTypeInfo;

    /// PyTypeObject instance for this type, which might still need to
    /// be initialized
    unsafe fn type_object() -> &'static mut ffi::PyTypeObject;

    /// Check if `*mut ffi::PyObject` is instance of this type
    fn is_instance(object: &PyAny) -> bool {
        unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object()) != 0 }
    }

    /// Check if `*mut ffi::PyObject` is exact instance of this type
    fn is_exact_instance(object: &PyAny) -> bool {
        unsafe { (*object.as_ptr()).ob_type == Self::type_object() }
    }
}

/// type object supports python GC
pub const PY_TYPE_FLAG_GC: usize = 1;

/// Type object supports python weak references
pub const PY_TYPE_FLAG_WEAKREF: usize = 1 << 1;

/// Type object can be used as the base type of another type
pub const PY_TYPE_FLAG_BASETYPE: usize = 1 << 2;

/// The instances of this type have a dictionary containing instance variables
pub const PY_TYPE_FLAG_DICT: usize = 1 << 3;

/// Special object that is used for python object creation.
/// `pyo3` library automatically creates this object for class `__new__` method.
/// Behavior is undefined if constructor of custom class does not initialze
/// instance of `PyRawObject` with rust value with `init` method.
/// Calling of `__new__` method of base class is developer's responsibility.
///
/// Example of custom class implementation with `__new__` method:
/// ```
/// use pyo3::prelude::*;
///
/// #[pyclass]
/// struct MyClass { }
///
/// #[pymethods]
/// impl MyClass {
///    #[new]
///    fn new(obj: &PyRawObject) {
///        obj.init(MyClass { })
///    }
/// }
/// ```
#[allow(dead_code)]
pub struct PyRawObject {
    ptr: *mut ffi::PyObject,
    /// Type object of class which __new__ method get called
    tp_ptr: *mut ffi::PyTypeObject,
    /// Type object of top most class in inheritance chain,
    /// it might be python class.
    curr_ptr: *mut ffi::PyTypeObject,
    // initialized: usize,
}

impl PyRawObject {
    #[must_use]
    pub unsafe fn new(
        py: Python,
        tp_ptr: *mut ffi::PyTypeObject,
        curr_ptr: *mut ffi::PyTypeObject,
    ) -> PyResult<PyRawObject> {
        let alloc = (*curr_ptr).tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
        let ptr = alloc(curr_ptr, 0);

        if !ptr.is_null() {
            Ok(PyRawObject {
                ptr,
                tp_ptr,
                curr_ptr,
                // initialized: 0,
            })
        } else {
            PyErr::fetch(py).into()
        }
    }

    #[must_use]
    pub unsafe fn new_with_ptr(
        py: Python,
        ptr: *mut ffi::PyObject,
        tp_ptr: *mut ffi::PyTypeObject,
        curr_ptr: *mut ffi::PyTypeObject,
    ) -> PyResult<PyRawObject> {
        if !ptr.is_null() {
            Ok(PyRawObject {
                ptr,
                tp_ptr,
                curr_ptr,
                // initialized: 0,
            })
        } else {
            PyErr::fetch(py).into()
        }
    }

    pub fn init<T: PyTypeInfo>(&self, value: T) {
        unsafe {
            // The `as *mut u8` part is required because the offset is in bytes
            let ptr = (self.ptr as *mut u8).offset(T::OFFSET) as *mut T;
            std::ptr::write(ptr, value);
        }
    }

    /// Type object
    pub fn type_object(&self) -> &PyType {
        unsafe { PyType::from_type_ptr(self.py(), self.curr_ptr) }
    }
}

impl<T: PyTypeInfo> AsRef<T> for PyRawObject {
    #[inline]
    fn as_ref(&self) -> &T {
        // TODO: check is object initialized
        unsafe {
            let ptr = (self.ptr as *mut u8).offset(T::OFFSET) as *mut T;
            ptr.as_ref().unwrap()
        }
    }
}

impl IntoPyPointer for PyRawObject {
    fn into_ptr(self) -> *mut ffi::PyObject {
        // TODO: panic if not all types initialized
        self.ptr
    }
}

unsafe impl PyNativeType for PyRawObject {}

pub(crate) unsafe fn pytype_drop<T: PyTypeInfo>(py: Python, obj: *mut ffi::PyObject) {
    if T::OFFSET != 0 {
        let ptr = (obj as *mut u8).offset(T::OFFSET) as *mut T;
        std::ptr::drop_in_place(ptr);
        pytype_drop::<T::BaseType>(py, obj);
    }
}

/// A Python object allocator that is usable as a base type for `#[pyclass]`
///
/// All native types and all `#[pyclass]` types use the default functions, while
/// [PyObjectWithFreeList](crate::freelist::PyObjectWithFreeList) gets a special version.
pub trait PyObjectAlloc: PyTypeInfo + Sized {
    unsafe fn alloc(_py: Python) -> *mut ffi::PyObject {
        let tp_ptr = Self::type_object();
        let alloc = (*tp_ptr).tp_alloc.unwrap_or(ffi::PyType_GenericAlloc);
        alloc(tp_ptr, 0)
    }

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject) {
        Self::drop(py, obj);

        if ffi::PyObject_CallFinalizerFromDealloc(obj) < 0 {
            return;
        }

        match Self::type_object().tp_free {
            Some(free) => free(obj as *mut c_void),
            None => {
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
        }
    }

    #[allow(unconditional_recursion)]
    /// Calls the rust destructor for the object.
    unsafe fn drop(py: Python, obj: *mut ffi::PyObject) {
        pytype_drop::<Self>(py, obj);
    }
}

/// Python object types that have a corresponding type object.
///
/// This trait is marked unsafe because not fulfilling the contract for [PyTypeObject::init_type]
/// leads to UB
pub unsafe trait PyTypeObject {
    /// This function must make sure that the corresponding type object gets
    /// initialized exactly once and return it.
    fn init_type() -> NonNull<ffi::PyTypeObject>;

    /// Returns the safe abstraction over the type object from [PyTypeObject::init_type]
    fn type_object() -> Py<PyType> {
        unsafe { Py::from_borrowed_ptr(Self::init_type().as_ptr() as *mut ffi::PyObject) }
    }
}

unsafe impl<T> PyTypeObject for T
where
    T: PyTypeInfo + PyMethodsProtocol + PyObjectAlloc,
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

/// Python object types that can be instanciated with [Self::create()]
///
/// We can't just make this a part of [PyTypeObject] because exceptions have
/// no PyTypeInfo
pub trait PyTypeCreate: PyObjectAlloc + PyTypeObject + Sized {
    /// Create PyRawObject which can be initialized with rust value
    #[must_use]
    fn create(py: Python) -> PyResult<PyRawObject> {
        Self::init_type();

        unsafe {
            let ptr = Self::alloc(py);
            PyRawObject::new_with_ptr(
                py,
                ptr,
                <Self as PyTypeInfo>::type_object(),
                <Self as PyTypeInfo>::type_object(),
            )
        }
    }
}

impl<T> PyTypeCreate for T where T: PyObjectAlloc + PyTypeObject + Sized {}

/// Register new type in python object system.
#[cfg(not(Py_LIMITED_API))]
pub fn initialize_type<T>(py: Python, module_name: Option<&str>) -> PyResult<*mut ffi::PyTypeObject>
where
    T: PyObjectAlloc + PyTypeInfo + PyMethodsProtocol,
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
    type_object.tp_dealloc = Some(tp_dealloc_callback::<T>);

    // type size
    type_object.tp_basicsize = <T as PyTypeInfo>::SIZE as ffi::Py_ssize_t;

    let mut offset = T::SIZE;
    // weakref support (check py3cls::py_class::impl_class)
    if T::FLAGS & PY_TYPE_FLAG_WEAKREF != 0 {
        offset -= std::mem::size_of::<*const ffi::PyObject>();
        type_object.tp_weaklistoffset = offset as isize;
    }

    // __dict__ support
    let has_dict = T::FLAGS & PY_TYPE_FLAG_DICT != 0;
    if has_dict {
        offset -= std::mem::size_of::<*const ffi::PyObject>();
        type_object.tp_dictoffset = offset as isize;
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
    let (new, init, call, mut methods) = py_class_method_defs::<T>();
    if !methods.is_empty() {
        methods.push(ffi::PyMethodDef_INIT);
        type_object.tp_methods = Box::into_raw(methods.into_boxed_slice()) as *mut _;
    }

    if let (None, Some(_)) = (new, init) {
        panic!(
            "{}.__new__ method is required if __init__ method defined",
            T::NAME
        );
    }

    // __new__ method
    type_object.tp_new = new;
    // __init__ method
    type_object.tp_init = init;
    // __call__ method
    type_object.tp_call = call;

    // properties
    let mut props = py_class_properties::<T>();

    if has_dict {
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

unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
where
    T: PyObjectAlloc,
{
    let _pool = gil::GILPool::new_no_pointers();
    let py = Python::assume_gil_acquired();
    <T as PyObjectAlloc>::dealloc(py, obj)
}
fn py_class_flags<T: PyTypeInfo>(type_object: &mut ffi::PyTypeObject) {
    if type_object.tp_traverse != None
        || type_object.tp_clear != None
        || T::FLAGS & PY_TYPE_FLAG_GC != 0
    {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC;
    } else {
        type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT;
    }
    if T::FLAGS & PY_TYPE_FLAG_BASETYPE != 0 {
        type_object.tp_flags |= ffi::Py_TPFLAGS_BASETYPE;
    }
}

fn py_class_method_defs<T: PyMethodsProtocol>() -> (
    Option<ffi::newfunc>,
    Option<ffi::initproc>,
    Option<ffi::PyCFunctionWithKeywords>,
    Vec<ffi::PyMethodDef>,
) {
    let mut defs = Vec::new();
    let mut call = None;
    let mut new = None;
    let mut init = None;

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
            PyMethodDefType::Init(ref def) => {
                if let class::methods::PyMethodType::PyInitFunc(meth) = def.ml_meth {
                    init = Some(meth)
                } else {
                    panic!("Method type is not supoorted by tp_init slot")
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

    (new, init, call, defs)
}

fn py_class_async_methods<T>(defs: &mut Vec<ffi::PyMethodDef>) {
    for def in <T as class::pyasync::PyAsyncProtocolImpl>::methods() {
        defs.push(def.as_method_def());
    }
}

fn py_class_properties<T: PyMethodsProtocol>() -> Vec<ffi::PyGetSetDef> {
    let mut defs = HashMap::new();

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
