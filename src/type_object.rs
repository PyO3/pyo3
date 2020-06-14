// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python type object information

use crate::once_cell::GILOnceCell;
use crate::pyclass::{initialize_type_object, PyClass};
use crate::pyclass_init::PyObjectInit;
use crate::types::{PyAny, PyType};
use crate::{ffi, AsPyPointer, PyNativeType, Python};
use parking_lot::{const_mutex, Mutex};
use std::thread::{self, ThreadId};

/// `T: PyLayout<U>` represents that `T` is a concrete representaion of `U` in Python heap.
/// E.g., `PyCell` is a concrete representaion of all `pyclass`es, and `ffi::PyObject`
/// is of `PyAny`.
///
/// This trait is intended to be used internally.
pub unsafe trait PyLayout<T: PyTypeInfo> {
    const IS_NATIVE_TYPE: bool = true;
    fn get_super(&mut self) -> Option<&mut T::BaseLayout> {
        None
    }
    unsafe fn py_init(&mut self, _value: T) {}
    unsafe fn py_drop(&mut self, _py: Python) {}
}

/// `T: PySizedLayout<U>` represents `T` is not a instance of
/// [`PyVarObject`](https://docs.python.org/3.8/c-api/structures.html?highlight=pyvarobject#c.PyVarObject).
/// , in addition that `T` is a concrete representaion of `U`.
pub trait PySizedLayout<T: PyTypeInfo>: PyLayout<T> + Sized {}

/// Marker type indicates that `Self` can be a base layout of `PyClass`.
///
/// # Safety
///
/// Self should be laid out as follows:
/// ```ignore
/// #[repr(C)]
/// struct Self {
///     obj: ffi::PyObject,
///     borrow_flag: u64,
///     ...
/// }
/// ```
/// Otherwise, implementing this trait is undefined behavior.
pub unsafe trait PyBorrowFlagLayout<T: PyTypeInfo>: PyLayout<T> + Sized {}

/// Our custom type flags
#[doc(hidden)]
pub mod type_flags {
    /// Type object supports Python GC
    pub const GC: usize = 1;

    /// Type object supports Python weak references
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
    type Layout: PyLayout<Self>;

    /// Layout of Basetype.
    type BaseLayout: PySizedLayout<Self::BaseType>;

    /// Initializer for layout
    type Initializer: PyObjectInit<Self>;

    /// Utility type to make AsPyRef work
    type AsRefTarget: crate::PyNativeType;

    /// PyTypeObject instance for this type.
    fn type_object_raw(py: Python) -> &'static ffi::PyTypeObject;

    /// Check if `*mut ffi::PyObject` is instance of this type
    fn is_instance(object: &PyAny) -> bool {
        unsafe {
            ffi::PyObject_TypeCheck(
                object.as_ptr(),
                Self::type_object(object.py()) as *const _ as _,
            ) != 0
        }
    }

    /// Check if `*mut ffi::PyObject` is exact instance of this type
    fn is_exact_instance(object: &PyAny) -> bool {
        unsafe { (*object.as_ptr()).ob_type == Self::type_object(object.py()) as *const _ as _ }
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
    fn type_object(py: Python) -> &PyType;
}

unsafe impl<T> PyTypeObject for T
where
    T: PyTypeInfo,
{
    fn type_object(py: Python) -> &PyType {
        unsafe { py.from_borrowed_ptr(Self::type_object_raw(py) as *const _ as _) }
    }
}

/// Lazy type object for PyClass
#[doc(hidden)]
pub struct LazyStaticType {
    // Boxed because Python expects the type object to have a stable address.
    value: GILOnceCell<Box<ffi::PyTypeObject>>,
    // Threads which have begun initialization. Used for reentrant initialization detection.
    initializing_threads: Mutex<Vec<ThreadId>>,
}

impl LazyStaticType {
    pub const fn new() -> Self {
        LazyStaticType {
            value: GILOnceCell::new(),
            initializing_threads: const_mutex(Vec::new()),
        }
    }

    pub fn get_or_init<T: PyClass>(&self, py: Python) -> &ffi::PyTypeObject {
        self.value
            .get_or_init(py, || {
                {
                    // Code evaluated at class init time, such as class attributes, might lead to
                    // recursive initalization of the type object if the class attribute is the same
                    // type as the class.
                    //
                    // That could lead to all sorts of unsafety such as using incomplete type objects
                    // to initialize class instances, so recursive initialization is prevented.
                    let thread_id = thread::current().id();
                    let mut threads = self.initializing_threads.lock();
                    if threads.contains(&thread_id) {
                        panic!("Recursive initialization of type_object for {}", T::NAME);
                    } else {
                        threads.push(thread_id)
                    }
                }

                // Okay, not recursive initialization - can proceed safely.
                let mut type_object = Box::new(ffi::PyTypeObject_INIT);

                initialize_type_object::<T>(py, T::MODULE, type_object.as_mut()).unwrap_or_else(
                    |e| {
                        e.print(py);
                        panic!("An error occurred while initializing class {}", T::NAME)
                    },
                );

                // Initialization successfully complete, can clear the thread list.
                // (No futher calls to get_or_init() will try to init, on any thread.)
                *self.initializing_threads.lock() = Vec::new();

                type_object
            })
            .as_ref()
    }
}

// This is necessary for making static `LazyStaticType`s
unsafe impl Sync for LazyStaticType {}
