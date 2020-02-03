// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python type object information

use crate::err::PyResult;
use crate::instance::Py;
use crate::pyclass::{create_type_object, PyClass};
use crate::pyclass_init::PyObjectInit;
use crate::types::{PyAny, PyType};
use crate::{ffi, AsPyPointer, Python};
use once_cell::sync::OnceCell;
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
#[doc(hidden)]
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
/// All Python native types(e.g., `PyDict`) and `#[pyclass]` structs implement this trait.
///
/// This trait is marked unsafe because:
///  - specifying the incorrect layout can lead to memory errors
///  - the return value of type_object must always point to the same PyTypeObject instance
pub unsafe trait PyTypeInfo: Sized {
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

    /// PyTypeObject instance for this type, guaranteed to be global and initialized.
    fn type_object() -> NonNull<ffi::PyTypeObject>;

    /// Check if `*mut ffi::PyObject` is instance of this type
    fn is_instance(object: &PyAny) -> bool {
        unsafe {
            ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object().as_ptr() as *mut _) != 0
        }
    }

    /// Check if `*mut ffi::PyObject` is exact instance of this type
    fn is_exact_instance(object: &PyAny) -> bool {
        unsafe { (*object.as_ptr()).ob_type == Self::type_object().as_ptr() as *mut _ }
    }
}

/// Python object types that have a corresponding type object.
///
/// This trait is marked unsafe because not fulfilling the contract for type_object
/// leads to UB.
///
/// See [PyTypeInfo::type_object]
pub unsafe trait PyTypeObject {
    /// Returns the safe abstraction over the type object.
    fn type_object() -> Py<PyType>;
}

unsafe impl<T> PyTypeObject for T
where
    T: PyTypeInfo,
{
    fn type_object() -> Py<PyType> {
        unsafe { Py::from_borrowed_ptr(<Self as PyTypeInfo>::type_object().as_ptr() as *mut _) }
    }
}

/// Type used to store static type objects
#[doc(hidden)]
pub struct LazyTypeObject {
    cell: OnceCell<NonNull<ffi::PyTypeObject>>,
}

impl LazyTypeObject {
    pub const fn new() -> Self {
        Self {
            cell: OnceCell::new(),
        }
    }

    pub fn get_or_init<F>(&self, constructor: F) -> PyResult<NonNull<ffi::PyTypeObject>>
    where
        F: Fn() -> PyResult<NonNull<ffi::PyTypeObject>>,
    {
        Ok(*self.cell.get_or_try_init(constructor)?)
    }

    pub fn get_pyclass_type<T: PyClass>(&self) -> NonNull<ffi::PyTypeObject> {
        self.get_or_init(|| {
            // automatically initialize the class on-demand
            let gil = Python::acquire_gil();
            let py = gil.python();
            let boxed = create_type_object::<T>(py, T::MODULE)?;
            Ok(unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) })
        })
        .unwrap_or_else(|e| {
            let gil = Python::acquire_gil();
            let py = gil.python();
            e.print(py);
            panic!("An error occurred while initializing class {}", T::NAME)
        })
    }
}

// This is necessary for making static `LazyTypeObject`s
//
// Type objects are shared between threads by the Python interpreter anyway, so it is no worse
// to allow sharing on the Rust side too.
unsafe impl Sync for LazyTypeObject {}
