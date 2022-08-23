#![allow(clippy::type_complexity)]
// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python type object information

use crate::impl_::pyclass::PyClassItems;
use crate::internal_tricks::extract_cstr_or_leak_cstring;
use crate::once_cell::GILOnceCell;
use crate::pyclass::create_type_object;
use crate::pyclass::PyClass;
use crate::types::{PyAny, PyType};
use crate::{conversion::IntoPyPointer, PyMethodDefType};
use crate::{ffi, AsPyPointer, PyNativeType, PyObject, PyResult, Python};
use parking_lot::{const_mutex, Mutex};
use std::thread::{self, ThreadId};

/// `T: PyLayout<U>` represents that `T` is a concrete representation of `U` in the Python heap.
/// E.g., `PyCell` is a concrete representaion of all `pyclass`es, and `ffi::PyObject`
/// is of `PyAny`.
///
/// This trait is intended to be used internally.
///
/// # Safety
///
/// This trait must only be implemented for types which represent valid layouts of Python objects.
pub unsafe trait PyLayout<T> {}

/// `T: PySizedLayout<U>` represents that `T` is not a instance of
/// [`PyVarObject`](https://docs.python.org/3.8/c-api/structures.html?highlight=pyvarobject#c.PyVarObject).
/// In addition, that `T` is a concrete representaion of `U`.
pub trait PySizedLayout<T>: PyLayout<T> + Sized {}

/// Python type information.
/// All Python native types (e.g., `PyDict`) and `#[pyclass]` structs implement this trait.
///
/// This trait is marked unsafe because:
///  - specifying the incorrect layout can lead to memory errors
///  - the return value of type_object must always point to the same PyTypeObject instance
///
/// It is safely implemented by the `pyclass` macro.
///
/// # Safety
///
/// Implementations must provide an implementation for `type_object_raw` which infallibly produces a
/// non-null pointer to the corresponding Python type object.
pub unsafe trait PyTypeInfo: Sized {
    /// Class name.
    const NAME: &'static str;

    /// Module name, if any.
    const MODULE: Option<&'static str>;

    /// Utility type to make Py::as_ref work.
    type AsRefTarget: PyNativeType;

    /// PyTypeObject instance for this type.
    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject;

    /// Checks if `object` is an instance of this type or a subclass of this type.
    fn is_type_of(object: &PyAny) -> bool {
        unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object_raw(object.py())) != 0 }
    }

    /// Checks if `object` is an instance of this type.
    fn is_exact_type_of(object: &PyAny) -> bool {
        unsafe { ffi::Py_TYPE(object.as_ptr()) == Self::type_object_raw(object.py()) }
    }
}

/// Python object types that have a corresponding type object.
///
/// # Safety
///
/// This trait is marked unsafe because not fulfilling the contract for type_object
/// leads to UB.
///
/// See also [PyTypeInfo::type_object_raw](trait.PyTypeInfo.html#tymethod.type_object_raw).
pub unsafe trait PyTypeObject {
    /// Returns the safe abstraction over the type object.
    fn type_object(py: Python<'_>) -> &PyType;
}

unsafe impl<T> PyTypeObject for T
where
    T: PyTypeInfo,
{
    fn type_object(py: Python<'_>) -> &PyType {
        unsafe { py.from_borrowed_ptr(Self::type_object_raw(py) as _) }
    }
}

/// Lazy type object for PyClass.
#[doc(hidden)]
pub struct LazyStaticType {
    // Boxed because Python expects the type object to have a stable address.
    value: GILOnceCell<*mut ffi::PyTypeObject>,
    // Threads which have begun initialization of the `tp_dict`. Used for
    // reentrant initialization detection.
    initializing_threads: Mutex<Vec<ThreadId>>,
    tp_dict_filled: GILOnceCell<PyResult<()>>,
}

impl LazyStaticType {
    pub const fn new() -> Self {
        LazyStaticType {
            value: GILOnceCell::new(),
            initializing_threads: const_mutex(Vec::new()),
            tp_dict_filled: GILOnceCell::new(),
        }
    }

    pub fn get_or_init<T: PyClass>(&self, py: Python<'_>) -> *mut ffi::PyTypeObject {
        let type_object = *self.value.get_or_init(py, || create_type_object::<T>(py));
        self.ensure_init(py, type_object, T::NAME, &T::for_all_items);
        type_object
    }

    fn ensure_init(
        &self,
        py: Python<'_>,
        type_object: *mut ffi::PyTypeObject,
        name: &str,
        for_all_items: &dyn Fn(&mut dyn FnMut(&PyClassItems)),
    ) {
        // We might want to fill the `tp_dict` with python instances of `T`
        // itself. In order to do so, we must first initialize the type object
        // with an empty `tp_dict`: now we can create instances of `T`.
        //
        // Then we fill the `tp_dict`. Multiple threads may try to fill it at
        // the same time, but only one of them will succeed.
        //
        // More importantly, if a thread is performing initialization of the
        // `tp_dict`, it can still request the type object through `get_or_init`,
        // but the `tp_dict` may appear empty of course.

        if self.tp_dict_filled.get(py).is_some() {
            // `tp_dict` is already filled: ok.
            return;
        }

        {
            let thread_id = thread::current().id();
            let mut threads = self.initializing_threads.lock();
            if threads.contains(&thread_id) {
                // Reentrant call: just return the type object, even if the
                // `tp_dict` is not filled yet.
                return;
            }
            threads.push(thread_id);
        }

        // Pre-compute the class attribute objects: this can temporarily
        // release the GIL since we're calling into arbitrary user code. It
        // means that another thread can continue the initialization in the
        // meantime: at worst, we'll just make a useless computation.
        let mut items = vec![];
        for_all_items(&mut |class_items| {
            items.extend(class_items.methods.iter().filter_map(|def| {
                if let PyMethodDefType::ClassAttribute(attr) = def {
                    let key = extract_cstr_or_leak_cstring(
                        attr.name,
                        "class attribute name cannot contain nul bytes",
                    )
                    .unwrap();

                    let val = (attr.meth.0)(py);
                    Some((key, val))
                } else {
                    None
                }
            }));
        });

        // Now we hold the GIL and we can assume it won't be released until we
        // return from the function.
        let result = self.tp_dict_filled.get_or_init(py, move || {
            let result = initialize_tp_dict(py, type_object as *mut ffi::PyObject, items);

            // Initialization successfully complete, can clear the thread list.
            // (No further calls to get_or_init() will try to init, on any thread.)
            *self.initializing_threads.lock() = Vec::new();
            result
        });

        if let Err(err) = result {
            err.clone_ref(py).print(py);
            panic!("An error occured while initializing `{}.__dict__`", name);
        }
    }
}

fn initialize_tp_dict(
    py: Python<'_>,
    type_object: *mut ffi::PyObject,
    items: Vec<(&'static std::ffi::CStr, PyObject)>,
) -> PyResult<()> {
    // We hold the GIL: the dictionary update can be considered atomic from
    // the POV of other threads.
    for (key, val) in items {
        let ret = unsafe { ffi::PyObject_SetAttrString(type_object, key.as_ptr(), val.into_ptr()) };
        crate::err::error_on_minusone(py, ret)?;
    }
    Ok(())
}

// This is necessary for making static `LazyStaticType`s
unsafe impl Sync for LazyStaticType {}

#[inline]
pub(crate) unsafe fn get_tp_alloc(tp: *mut ffi::PyTypeObject) -> Option<ffi::allocfunc> {
    #[cfg(not(Py_LIMITED_API))]
    {
        (*tp).tp_alloc
    }

    #[cfg(Py_LIMITED_API)]
    {
        let ptr = ffi::PyType_GetSlot(tp, ffi::Py_tp_alloc);
        std::mem::transmute(ptr)
    }
}

#[inline]
pub(crate) unsafe fn get_tp_free(tp: *mut ffi::PyTypeObject) -> ffi::freefunc {
    #[cfg(not(Py_LIMITED_API))]
    {
        (*tp).tp_free.unwrap()
    }

    #[cfg(Py_LIMITED_API)]
    {
        let ptr = ffi::PyType_GetSlot(tp, ffi::Py_tp_free);
        debug_assert_ne!(ptr, std::ptr::null_mut());
        std::mem::transmute(ptr)
    }
}
