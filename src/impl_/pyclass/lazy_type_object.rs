use std::{
    ffi::CStr,
    marker::PhantomData,
    thread::{self, ThreadId},
};

#[cfg(Py_3_14)]
use crate::err::error_on_minusone;
#[allow(deprecated)]
use crate::sync::GILOnceCell;
#[cfg(Py_3_14)]
use crate::types::PyTypeMethods;
use crate::{
    exceptions::PyRuntimeError,
    ffi,
    impl_::{pyclass::MaybeRuntimePyMethodDef, pymethods::PyMethodDefType},
    pyclass::{create_type_object, PyClassTypeObject},
    types::PyType,
    Bound, Py, PyAny, PyClass, PyErr, PyResult, Python,
};

use std::sync::Mutex;

use super::PyClassItemsIter;

/// Lazy type object for PyClass.
#[doc(hidden)]
pub struct LazyTypeObject<T>(LazyTypeObjectInner, PhantomData<T>);

// Non-generic inner of LazyTypeObject to keep code size down
struct LazyTypeObjectInner {
    #[allow(deprecated)]
    value: GILOnceCell<PyClassTypeObject>,
    // Threads which have begun initialization of the `tp_dict`. Used for
    // reentrant initialization detection.
    initializing_threads: Mutex<Vec<ThreadId>>,
    #[allow(deprecated)]
    fully_initialized_type: GILOnceCell<Py<PyType>>,
}

impl<T> LazyTypeObject<T> {
    /// Creates an uninitialized `LazyTypeObject`.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        LazyTypeObject(
            LazyTypeObjectInner {
                #[allow(deprecated)]
                value: GILOnceCell::new(),
                initializing_threads: Mutex::new(Vec::new()),
                #[allow(deprecated)]
                fully_initialized_type: GILOnceCell::new(),
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
        self.0
            .get_or_try_init(py, create_type_object::<T>, T::NAME, T::items_iter())
    }
}

impl LazyTypeObjectInner {
    // Uses dynamically dispatched fn(Python<'py>) -> PyResult<Py<PyType>
    // so that this code is only instantiated once, instead of for every T
    // like the generic LazyTypeObject<T> methods above.
    fn get_or_try_init<'py>(
        &self,
        py: Python<'py>,
        init: fn(Python<'py>) -> PyResult<PyClassTypeObject>,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<&Bound<'py, PyType>> {
        (|| -> PyResult<_> {
            let PyClassTypeObject {
                type_object,
                is_immutable_type,
                ..
            } = self.value.get_or_try_init(py, || init(py))?;
            let type_object = type_object.bind(py);
            self.ensure_init(type_object, *is_immutable_type, name, items_iter)?;
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

    fn ensure_init(
        &self,
        type_object: &Bound<'_, PyType>,
        #[allow(unused_variables)] is_immutable_type: bool,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<()> {
        let py = type_object.py();

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

        if self.fully_initialized_type.get(py).is_some() {
            // `tp_dict` is already filled: ok.
            return Ok(());
        }

        let thread_id = thread::current().id();
        {
            let mut threads = self.initializing_threads.lock().unwrap();
            if threads.contains(&thread_id) {
                // Reentrant call: just return the type object, even if the
                // `tp_dict` is not filled yet.
                return Ok(());
            }
            threads.push(thread_id);
        }

        struct InitializationGuard<'a> {
            initializing_threads: &'a Mutex<Vec<ThreadId>>,
            thread_id: ThreadId,
        }
        impl Drop for InitializationGuard<'_> {
            fn drop(&mut self) {
                let mut threads = self.initializing_threads.lock().unwrap();
                threads.retain(|id| *id != self.thread_id);
            }
        }

        let guard = InitializationGuard {
            initializing_threads: &self.initializing_threads,
            thread_id,
        };

        // Pre-compute the class attribute objects: this can temporarily
        // release the GIL since we're calling into arbitrary user code. It
        // means that another thread can continue the initialization in the
        // meantime: at worst, we'll just make a useless computation.
        let mut items = vec![];
        for class_items in items_iter {
            for def in class_items.methods {
                let built_method;
                let method = match def {
                    MaybeRuntimePyMethodDef::Runtime(builder) => {
                        built_method = builder();
                        &built_method
                    }
                    MaybeRuntimePyMethodDef::Static(method) => method,
                };
                if let PyMethodDefType::ClassAttribute(attr) = method {
                    match (attr.meth)(py) {
                        Ok(val) => items.push((attr.name, val)),
                        Err(err) => {
                            return Err(wrap_in_runtime_error(
                                py,
                                err,
                                format!(
                                    "An error occurred while initializing `{}.{}`",
                                    name,
                                    attr.name.to_str().unwrap()
                                ),
                            ))
                        }
                    }
                }
            }
        }

        // Now we hold the GIL and we can assume it won't be released until we
        // return from the function.
        let result = self.fully_initialized_type.get_or_try_init(py, move || {
            initialize_tp_dict(py, type_object.as_ptr(), items)?;
            #[cfg(Py_3_14)]
            if is_immutable_type {
                // freeze immutable types after __dict__ is initialized
                let res = unsafe { ffi::PyType_Freeze(type_object.as_type_ptr()) };
                error_on_minusone(py, res)?;
            }
            #[cfg(all(Py_3_10, not(Py_LIMITED_API), not(Py_3_14)))]
            if is_immutable_type {
                use crate::types::PyTypeMethods as _;
                #[cfg(not(Py_GIL_DISABLED))]
                unsafe {
                    (*type_object.as_type_ptr()).tp_flags |= ffi::Py_TPFLAGS_IMMUTABLETYPE
                };
                #[cfg(Py_GIL_DISABLED)]
                unsafe {
                    (*type_object.as_type_ptr()).tp_flags.fetch_or(
                        ffi::Py_TPFLAGS_IMMUTABLETYPE,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                };
                unsafe { ffi::PyType_Modified(type_object.as_type_ptr()) };
            }

            // Initialization successfully complete, can clear the thread list.
            // (No further calls to get_or_init() will try to init, on any thread.)
            let mut threads = {
                drop(guard);
                self.initializing_threads.lock().unwrap()
            };
            threads.clear();
            Ok(type_object.clone().unbind())
        });

        if let Err(err) = result {
            return Err(wrap_in_runtime_error(
                py,
                err,
                format!("An error occurred while initializing `{name}.__dict__`"),
            ));
        }

        Ok(())
    }
}

fn initialize_tp_dict(
    py: Python<'_>,
    type_object: *mut ffi::PyObject,
    items: Vec<(&'static CStr, Py<PyAny>)>,
) -> PyResult<()> {
    // We hold the GIL: the dictionary update can be considered atomic from
    // the POV of other threads.
    for (key, val) in items {
        crate::err::error_on_minusone(py, unsafe {
            ffi::PyObject_SetAttrString(type_object, key.as_ptr(), val.into_ptr())
        })?;
    }
    Ok(())
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
