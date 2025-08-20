use std::{
    marker::PhantomData,
    thread::{self, ThreadId},
};

use pyo3_ffi::PyTypeObject;

#[cfg(Py_3_14)]
use crate::err::error_on_minusone;
#[cfg(Py_3_14)]
use crate::types::PyTypeMethods;
use crate::{
    exceptions::PyRuntimeError,
    impl_::pymethods::PyMethodDefType,
    pyclass::{create_type_object, PyClassTypeObject},
    sync::PyOnceLock,
    type_object::PyTypeInfo,
    types::{PyAnyMethods, PyType},
    Bound, Py, PyClass, PyErr, PyResult, Python,
};

use std::sync::Mutex;

use super::PyClassItemsIter;

/// Lazy type object for PyClass.
#[doc(hidden)]
pub struct LazyTypeObject<T>(LazyTypeObjectInner, PhantomData<T>);

// Non-generic inner of LazyTypeObject to keep code size down
struct LazyTypeObjectInner {
    value: PyOnceLock<PyClassTypeObject>,
    initializing_thread: Mutex<Option<ThreadId>>,
    fully_initialized_type: PyOnceLock<Py<PyType>>,
}

impl<T> LazyTypeObject<T> {
    /// Creates an uninitialized `LazyTypeObject`.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        LazyTypeObject(
            LazyTypeObjectInner {
                value: PyOnceLock::new(),
                initializing_thread: Mutex::new(None),
                fully_initialized_type: PyOnceLock::new(),
            },
            PhantomData,
        )
    }
}

impl<T: PyClass> LazyTypeObject<T> {
    /// Gets the type object contained by this `LazyTypeObject`, initializing it if needed.
    #[inline]
    pub fn get_or_try_init<'py>(&self, py: Python<'py>) -> PyResult<&Bound<'py, PyType>> {
        if let Some(type_object) = self.0.fully_initialized_type.get(py) {
            // Fast path
            return Ok(type_object.bind(py));
        }

        self.try_init(py)
    }

    #[cold]
    fn try_init<'py>(&self, py: Python<'py>) -> PyResult<&Bound<'py, PyType>> {
        self.0.get_or_try_init(
            py,
            <T::BaseType as PyTypeInfo>::type_object_raw,
            create_type_object::<T>,
            T::NAME,
            T::items_iter(),
        )
    }
}

impl LazyTypeObjectInner {
    // Uses dynamically dispatched fn(Python<'py>) -> PyResult<Py<PyType>
    // so that this code is only instantiated once, instead of for every T
    // like the generic LazyTypeObject<T> methods above.
    fn get_or_try_init<'py>(
        &self,
        py: Python<'py>,
        base_init: fn(Python<'py>) -> *mut PyTypeObject,
        init: fn(Python<'py>) -> PyResult<PyClassTypeObject>,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<&Bound<'py, PyType>> {
        (|| -> PyResult<_> {
            // ensure that base is fully initialized before entering the `PyOnceLock`
            // initialization; that could otherwise deadlock if the base type needs
            // to load the subtype as an attribute.
            //
            // don't try to synchronize this; assume that `base_init` handles concurrency and
            // re-entrancy in the same way this function does
            base_init(py);
            // at this point, we are guaranteed that the base type object has been created, we may be inside
            // `fill_tp_dict` of the base type in the case of this subtype being an attribute on the base
            let PyClassTypeObject {
                type_object,
                is_immutable_type,
                ..
            } = self.value.get_or_try_init(py, || init(py))?;
            let type_object = type_object.bind(py);
            self.fill_tp_dict(type_object, *is_immutable_type, name, items_iter)?;
            Ok(type_object)
        })()
        .map_err(|err| {
            wrap_in_runtime_error(
                py,
                err,
                format!("An error occurred while initializing class {name}"),
            )
        })
    }

    fn fill_tp_dict(
        &self,
        type_object: &Bound<'_, PyType>,
        #[allow(unused_variables)] is_immutable_type: bool,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<()> {
        let py: Python<'_> = type_object.py();

        // We might want to fill the `tp_dict` with python instances of `T`
        // itself. In order to do so, we must first initialize the type object
        // with an empty `tp_dict`: now we can create instances of `T`.
        //
        // More importantly, if a thread is performing initialization of the
        // `tp_dict`, it can still request the type object through `get_or_init`,
        // but the `tp_dict` may appear empty of course.

        let Some(guard) = InitializationGuard::new(&self.initializing_thread) else {
            // we are re-entrant with `tp_dict` initialization on this thread, we should
            // just return Ok and allow the init to proceed, whatever is accessing the type
            // object will just see the class without all attributes present.
            return Ok(());
        };

        // Only one thread will now proceed to set the type attributes.
        self.fully_initialized_type
            .get_or_try_init(py, move || -> PyResult<_> {
                guard.start_init();

                for class_items in items_iter {
                    for method in class_items.methods {
                        if let PyMethodDefType::ClassAttribute(attr) = method {
                            (attr.meth)(py)
                                .and_then(|val| {
                                    type_object.setattr(
                                        // FIXME: add `IntoPyObject` for `&CStr`?
                                        attr.name.to_str().expect("attribute name should be UTF8"),
                                        val,
                                    )
                                })
                                .map_err(|err| {
                                    wrap_in_runtime_error(
                                        py,
                                        err,
                                        format!(
                                            "An error occurred while initializing `{}.{}`",
                                            name,
                                            attr.name.to_str().unwrap()
                                        ),
                                    )
                                })?;
                        }
                    }
                }

                #[cfg(Py_3_14)]
                if is_immutable_type {
                    // freeze immutable types after __dict__ is initialized
                    let res = unsafe { crate::ffi::PyType_Freeze(type_object.as_type_ptr()) };
                    error_on_minusone(py, res)?;
                }
                #[cfg(all(Py_3_10, not(Py_LIMITED_API), not(Py_3_14)))]
                if is_immutable_type {
                    use crate::types::PyTypeMethods as _;
                    #[cfg(not(Py_GIL_DISABLED))]
                    unsafe {
                        (*type_object.as_type_ptr()).tp_flags |=
                            crate::ffi::Py_TPFLAGS_IMMUTABLETYPE
                    };
                    #[cfg(Py_GIL_DISABLED)]
                    unsafe {
                        (*type_object.as_type_ptr()).tp_flags.fetch_or(
                            crate::ffi::Py_TPFLAGS_IMMUTABLETYPE,
                            std::sync::atomic::Ordering::Relaxed,
                        )
                    };
                    unsafe { crate::ffi::PyType_Modified(type_object.as_type_ptr()) };
                }

                drop(guard);
                Ok(type_object.clone().unbind())
            })?;

        Ok(())
    }
}

struct InitializationGuard<'a> {
    initializing_thread: &'a Mutex<Option<ThreadId>>,
    thread_id: ThreadId,
}

impl<'a> InitializationGuard<'a> {
    /// Attempt to create a new `InitializationGuard`.
    ///
    /// Returns `None` if this call would be re-entrant.
    ///
    /// The guard will not protect against re-entrancy until `start_init` is called.
    fn new(initializing_thread: &'a Mutex<Option<ThreadId>>) -> Option<Self> {
        let thread_id = thread::current().id();
        let thread = initializing_thread.lock().expect("no poisoning");
        if thread.is_some_and(|id| id == thread_id) {
            None
        } else {
            Some(Self {
                initializing_thread,
                thread_id,
            })
        }
    }

    /// Starts the initialization process. From this point forward `InitializationGuard::new` will protect against re-entrancy.
    fn start_init(&self) {
        let mut thread = self.initializing_thread.lock().expect("no poisoning");
        assert!(thread.is_none(), "overlapping use of `InitializationGuard`");
        *thread = Some(self.thread_id);
    }
}

impl Drop for InitializationGuard<'_> {
    fn drop(&mut self) {
        let mut thread = self.initializing_thread.lock().unwrap();
        // only clear the thread if this was the thread which called `start_init`
        if thread.is_some_and(|id| id == self.thread_id) {
            *thread = None;
        }
    }
}

// This is necessary for making static `LazyTypeObject`s
unsafe impl<T> Sync for LazyTypeObject<T> {}

/// Used in the macro-expanded implementation of `type_object_raw` for `#[pyclass]` types
#[cold]
pub fn type_object_init_failed(py: Python<'_>, err: PyErr, type_name: &str) -> ! {
    err.write_unraisable(py, None);
    panic!("failed to create type object for `{type_name}`")
}

#[cold]
fn wrap_in_runtime_error(py: Python<'_>, err: PyErr, message: String) -> PyErr {
    let runtime_err = PyRuntimeError::new_err(message);
    runtime_err.set_cause(py, Some(err));
    runtime_err
}
