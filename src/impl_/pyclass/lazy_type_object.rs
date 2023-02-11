use std::{
    borrow::Cow,
    ffi::CStr,
    marker::PhantomData,
    thread::{self, ThreadId},
};

use parking_lot::{const_mutex, Mutex};

use crate::{
    exceptions::PyRuntimeError, ffi, once_cell::GILOnceCell, pyclass::create_type_object,
    IntoPyPointer, PyClass, PyErr, PyMethodDefType, PyObject, PyResult, Python,
};

use super::PyClassItemsIter;

/// Lazy type object for PyClass.
#[doc(hidden)]
pub struct LazyTypeObject<T>(LazyTypeObjectInner, PhantomData<T>);

// Non-generic inner of LazyTypeObject to keep code size down
struct LazyTypeObjectInner {
    value: GILOnceCell<*mut ffi::PyTypeObject>,
    // Threads which have begun initialization of the `tp_dict`. Used for
    // reentrant initialization detection.
    initializing_threads: Mutex<Vec<ThreadId>>,
    tp_dict_filled: GILOnceCell<PyResult<()>>,
}

impl<T> LazyTypeObject<T> {
    /// Creates an uninitialized `LazyTypeObject`.
    pub const fn new() -> Self {
        LazyTypeObject(
            LazyTypeObjectInner {
                value: GILOnceCell::new(),
                initializing_threads: const_mutex(Vec::new()),
                tp_dict_filled: GILOnceCell::new(),
            },
            PhantomData,
        )
    }
}

impl<T: PyClass> LazyTypeObject<T> {
    /// Gets the type object contained by this `LazyTypeObject`, initializing it if needed.
    pub fn get_or_init(&self, py: Python<'_>) -> *mut ffi::PyTypeObject {
        self.get_or_try_init(py).unwrap_or_else(|err| {
            err.print(py);
            panic!("failed to create type object for {}", T::NAME)
        })
    }

    /// Fallible version of the above.
    pub(crate) fn get_or_try_init(&self, py: Python<'_>) -> PyResult<*mut ffi::PyTypeObject> {
        fn inner<T: PyClass>() -> PyResult<*mut ffi::PyTypeObject> {
            // Safety: `py` is held by the caller of `get_or_init`.
            let py = unsafe { Python::assume_gil_acquired() };
            create_type_object::<T>(py)
        }

        self.0
            .get_or_try_init(py, inner::<T>, T::NAME, T::items_iter())
    }
}

impl LazyTypeObjectInner {
    fn get_or_try_init(
        &self,
        py: Python<'_>,
        init: fn() -> PyResult<*mut ffi::PyTypeObject>,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<*mut ffi::PyTypeObject> {
        (|| -> PyResult<_> {
            // Uses explicit GILOnceCell::get_or_init::<fn() -> *mut ffi::PyTypeObject> monomorphization
            // so that this code is only monomorphized once, instead of for every T.
            let type_object = *self.value.get_or_try_init(py, init)?;
            self.ensure_init(py, type_object, name, items_iter)?;
            Ok(type_object)
        })()
        .map_err(|err| {
            wrap_in_runtime_error(
                py,
                err,
                format!("An error occurred while initializing class {}", name),
            )
        })
    }

    fn ensure_init(
        &self,
        py: Python<'_>,
        type_object: *mut ffi::PyTypeObject,
        name: &str,
        items_iter: PyClassItemsIter,
    ) -> PyResult<()> {
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
            return Ok(());
        }

        let thread_id = thread::current().id();
        {
            let mut threads = self.initializing_threads.lock();
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
                let mut threads = self.initializing_threads.lock();
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
                if let PyMethodDefType::ClassAttribute(attr) = def {
                    let key = attr.attribute_c_string().unwrap();

                    match (attr.meth.0)(py) {
                        Ok(val) => items.push((key, val)),
                        Err(err) => {
                            return Err(wrap_in_runtime_error(
                                py,
                                err,
                                format!(
                                    "An error occurred while initializing `{}.{}`",
                                    name,
                                    attr.name.trim_end_matches('\0')
                                ),
                            ))
                        }
                    }
                }
            }
        }

        // Now we hold the GIL and we can assume it won't be released until we
        // return from the function.
        let result = self.tp_dict_filled.get_or_init(py, move || {
            let result = initialize_tp_dict(py, type_object as *mut ffi::PyObject, items);

            // Initialization successfully complete, can clear the thread list.
            // (No further calls to get_or_init() will try to init, on any thread.)
            std::mem::forget(guard);
            *self.initializing_threads.lock() = Vec::new();
            result
        });

        if let Err(err) = result {
            return Err(wrap_in_runtime_error(
                py,
                err.clone_ref(py),
                format!("An error occurred while initializing `{}.__dict__`", name),
            ));
        }

        Ok(())
    }
}

fn initialize_tp_dict(
    py: Python<'_>,
    type_object: *mut ffi::PyObject,
    items: Vec<(Cow<'static, CStr>, PyObject)>,
) -> PyResult<()> {
    // We hold the GIL: the dictionary update can be considered atomic from
    // the POV of other threads.
    for (key, val) in items {
        let ret = unsafe { ffi::PyObject_SetAttrString(type_object, key.as_ptr(), val.into_ptr()) };
        crate::err::error_on_minusone(py, ret)?;
    }
    Ok(())
}

// This is necessary for making static `LazyTypeObject`s
unsafe impl<T> Sync for LazyTypeObject<T> {}

#[cold]
fn wrap_in_runtime_error(py: Python<'_>, err: PyErr, message: String) -> PyErr {
    let runtime_err = PyRuntimeError::new_err(message);
    runtime_err.set_cause(py, Some(err));
    runtime_err
}
