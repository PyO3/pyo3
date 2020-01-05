// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python type object information

use crate::instance::Py;
use crate::pyclass_init::PyObjectInit;
use crate::types::{PyAny, PyType};
use crate::{ffi, AsPyPointer, Python};
use std::ptr::NonNull;

/// `T: PyObjectLayout<U>` represents that `T` is a concrete representaion of `U` in Python heap.
/// E.g., `PyClassShell` is a concrete representaion of all `pyclass`es, and `ffi::PyObject`
/// is of `PyAny`.
///
/// This trait is intended to be used internally.
pub trait PyObjectLayout<T: PyTypeInfo> {
    const IS_NATIVE_TYPE: bool = true;

    fn get_super_or(&mut self) -> Option<&mut <T::BaseType as PyTypeInfo>::ConcreteLayout> {
        None
    }

    unsafe fn internal_ref_cast(obj: &PyAny) -> &T {
        &*(obj as *const _ as *const T)
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn internal_mut_cast(obj: &PyAny) -> &mut T {
        &mut *(obj as *const _ as *const T as *mut T)
    }

    unsafe fn py_init(&mut self, _value: T) {}
    unsafe fn py_drop(&mut self, _py: Python) {}
}

/// `T: PyObjectSizedLayout<U>` represents `T` is not a instance of
/// [`PyVarObject`](https://docs.python.org/3.8/c-api/structures.html?highlight=pyvarobject#c.PyVarObject).
/// , in addition that `T` is a concrete representaion of `U`.
///
/// `pyclass`es need this trait for their base class.
pub trait PyObjectSizedLayout<T: PyTypeInfo>: PyObjectLayout<T> + Sized {}

/// Our custom type flags
pub mod type_flags {
    /// type object supports python GC
    pub const GC: usize = 1;

    /// Type object supports python weak references
    pub const WEAKREF: usize = 1 << 1;

    /// Type object can be used as the base type of another type
    pub const BASETYPE: usize = 1 << 2;

    /// The instances of this type have a dictionary containing instance variables
    pub const DICT: usize = 1 << 3;

    /// The class declared by #[pyclass(extends=~)]
    pub const EXTENDED: usize = 1 << 4;
}

/// Python type information.
pub trait PyTypeInfo: Sized {
    /// Type of objects to store in PyObject struct
    type Type;

    /// Class name
    const NAME: &'static str;

    /// Module name, if any
    const MODULE: Option<&'static str>;

    /// Class doc string
    const DESCRIPTION: &'static str = "\0";

    /// Type flags (ie PY_TYPE_FLAG_GC, PY_TYPE_FLAG_WEAKREF)
    const FLAGS: usize = 0;

    /// Base class
    type BaseType: PyTypeInfo + PyTypeObject;

    /// Layout
    type ConcreteLayout: PyObjectLayout<Self>;

    /// Initializer for layout
    type Initializer: PyObjectInit<Self>;

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
