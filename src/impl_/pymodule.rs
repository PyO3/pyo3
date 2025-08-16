//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::{cell::UnsafeCell, ffi::CStr, marker::PhantomData};

#[cfg(all(
    not(any(PyPy, GraalPy)),
    Py_3_9,
    not(all(windows, Py_LIMITED_API, not(Py_3_10))),
    not(target_has_atomic = "64"),
))]
use portable_atomic::AtomicI64;
#[cfg(all(
    not(any(PyPy, GraalPy)),
    Py_3_9,
    not(all(windows, Py_LIMITED_API, not(Py_3_10))),
    target_has_atomic = "64",
))]
use std::sync::atomic::AtomicI64;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(any(PyPy, GraalPy)))]
use crate::exceptions::PyImportError;
#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
use crate::PyErr;
use crate::{
    ffi,
    impl_::pymethods::PyMethodDef,
    sync::PyOnceLock,
    types::{PyCFunction, PyModule, PyModuleMethods},
    Bound, Py, PyClass, PyResult, PyTypeInfo, Python,
};

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
    initializer: ModuleInitializer,
    /// Interpreter ID where module was initialized (not applicable on PyPy).
    #[cfg(all(
        not(any(PyPy, GraalPy)),
        Py_3_9,
        not(all(windows, Py_LIMITED_API, not(Py_3_10)))
    ))]
    interpreter: AtomicI64,
    /// Initialized module object, cached to avoid reinitialization.
    module: PyOnceLock<Py<PyModule>>,
    /// Whether or not the module supports running without the GIL
    gil_used: AtomicBool,
}

/// Wrapper to enable initializer to be used in const fns.
pub struct ModuleInitializer(pub for<'py> fn(&Bound<'py, PyModule>) -> PyResult<()>);

unsafe impl Sync for ModuleDef {}

impl ModuleDef {
    /// Make new module definition with given module name.
    pub const unsafe fn new(
        name: &'static CStr,
        doc: &'static CStr,
        initializer: ModuleInitializer,
    ) -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const INIT: ffi::PyModuleDef = ffi::PyModuleDef {
            m_base: ffi::PyModuleDef_HEAD_INIT,
            m_name: std::ptr::null(),
            m_doc: std::ptr::null(),
            m_size: 0,
            m_methods: std::ptr::null_mut(),
            m_slots: std::ptr::null_mut(),
            m_traverse: None,
            m_clear: None,
            m_free: None,
        };

        let ffi_def = UnsafeCell::new(ffi::PyModuleDef {
            m_name: name.as_ptr(),
            m_doc: doc.as_ptr(),
            ..INIT
        });

        ModuleDef {
            ffi_def,
            initializer,
            // -1 is never expected to be a valid interpreter ID
            #[cfg(all(
                not(any(PyPy, GraalPy)),
                Py_3_9,
                not(all(windows, Py_LIMITED_API, not(Py_3_10)))
            ))]
            interpreter: AtomicI64::new(-1),
            module: PyOnceLock::new(),
            gil_used: AtomicBool::new(true),
        }
    }
    /// Builds a module using user given initializer. Used for [`#[pymodule]`][crate::pymodule].
    #[cfg_attr(any(Py_LIMITED_API, not(Py_GIL_DISABLED)), allow(unused_variables))]
    pub fn make_module(&'static self, py: Python<'_>, gil_used: bool) -> PyResult<Py<PyModule>> {
        // Check the interpreter ID has not changed, since we currently have no way to guarantee
        // that static data is not reused across interpreters.
        //
        // PyPy does not have subinterpreters, so no need to check interpreter ID.
        #[cfg(not(any(PyPy, GraalPy)))]
        {
            // PyInterpreterState_Get is only available on 3.9 and later, but is missing
            // from python3.dll for Windows stable API on 3.9
            #[cfg(all(Py_3_9, not(all(windows, Py_LIMITED_API, not(Py_3_10)))))]
            {
                let current_interpreter =
                    unsafe { ffi::PyInterpreterState_GetID(ffi::PyInterpreterState_Get()) };
                crate::err::error_on_minusone(py, current_interpreter)?;
                if let Err(initialized_interpreter) = self.interpreter.compare_exchange(
                    -1,
                    current_interpreter,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    if initialized_interpreter != current_interpreter {
                        return Err(PyImportError::new_err(
                            "PyO3 modules do not yet support subinterpreters, see https://github.com/PyO3/pyo3/issues/576",
                        ));
                    }
                }
            }
            #[cfg(not(all(Py_3_9, not(all(windows, Py_LIMITED_API, not(Py_3_10))))))]
            {
                // CPython before 3.9 does not have APIs to check the interpreter ID, so best that can be
                // done to guard against subinterpreters is fail if the module is initialized twice
                if self.module.get(py).is_some() {
                    return Err(PyImportError::new_err(
                        "PyO3 modules compiled for CPython 3.8 or older may only be initialized once per interpreter process"
                    ));
                }
            }
        }
        self.module
            .get_or_try_init(py, || {
                let module = unsafe {
                    Py::<PyModule>::from_owned_ptr_or_err(
                        py,
                        ffi::PyModule_Create(self.ffi_def.get()),
                    )?
                };
                #[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
                {
                    let gil_used_ptr = {
                        if gil_used {
                            ffi::Py_MOD_GIL_USED
                        } else {
                            ffi::Py_MOD_GIL_NOT_USED
                        }
                    };
                    if unsafe { ffi::PyUnstable_Module_SetGIL(module.as_ptr(), gil_used_ptr) } < 0 {
                        return Err(PyErr::fetch(py));
                    }
                }
                self.initializer.0(module.bind(py))?;
                Ok(module)
            })
            .map(|py_module| py_module.clone_ref(py))
    }
}

/// Trait to add an element (class, function...) to a module.
///
/// Currently only implemented for classes.
pub trait PyAddToModule: crate::sealed::Sealed {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()>;
}

/// For adding native types (non-pyclass) to a module.
pub struct AddTypeToModule<T>(PhantomData<T>);

impl<T> AddTypeToModule<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        AddTypeToModule(PhantomData)
    }
}

impl<T: PyTypeInfo> PyAddToModule for AddTypeToModule<T> {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add(T::NAME, T::type_object(module.py()))
    }
}

/// For adding a class to a module.
pub struct AddClassToModule<T>(PhantomData<T>);

impl<T> AddClassToModule<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        AddClassToModule(PhantomData)
    }
}

impl<T: PyClass> PyAddToModule for AddClassToModule<T> {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_class::<T>()
    }
}

/// For adding a function to a module.
impl PyAddToModule for PyMethodDef {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(PyCFunction::internal_new(module.py(), self, Some(module))?)
    }
}

/// For adding a module to a module.
impl PyAddToModule for ModuleDef {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_submodule(
            self.make_module(module.py(), self.gil_used.load(Ordering::Relaxed))?
                .bind(module.py()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        ffi::CStr,
        sync::atomic::{AtomicBool, Ordering},
    };

    use crate::{
        ffi,
        types::{any::PyAnyMethods, module::PyModuleMethods, PyModule},
        Bound, PyResult, Python,
    };

    use super::{ModuleDef, ModuleInitializer};

    #[test]
    fn module_init() {
        static MODULE_DEF: ModuleDef = unsafe {
            ModuleDef::new(
                ffi::c_str!("test_module"),
                ffi::c_str!("some doc"),
                ModuleInitializer(|m| {
                    m.add("SOME_CONSTANT", 42)?;
                    Ok(())
                }),
            )
        };
        Python::attach(|py| {
            let module = MODULE_DEF.make_module(py, false).unwrap().into_bound(py);
            assert_eq!(
                module
                    .getattr("__name__")
                    .unwrap()
                    .extract::<Cow<'_, str>>()
                    .unwrap(),
                "test_module",
            );
            assert_eq!(
                module
                    .getattr("__doc__")
                    .unwrap()
                    .extract::<Cow<'_, str>>()
                    .unwrap(),
                "some doc",
            );
            assert_eq!(
                module
                    .getattr("SOME_CONSTANT")
                    .unwrap()
                    .extract::<u8>()
                    .unwrap(),
                42,
            );
        })
    }

    #[test]
    fn module_def_new() {
        // To get coverage for ModuleDef::new() need to create a non-static ModuleDef, however init
        // etc require static ModuleDef, so this test needs to be separated out.
        static NAME: &CStr = ffi::c_str!("test_module");
        static DOC: &CStr = ffi::c_str!("some doc");

        static INIT_CALLED: AtomicBool = AtomicBool::new(false);

        #[allow(clippy::unnecessary_wraps)]
        fn init(_: &Bound<'_, PyModule>) -> PyResult<()> {
            INIT_CALLED.store(true, Ordering::SeqCst);
            Ok(())
        }

        unsafe {
            let module_def: ModuleDef = ModuleDef::new(NAME, DOC, ModuleInitializer(init));
            assert_eq!((*module_def.ffi_def.get()).m_name, NAME.as_ptr() as _);
            assert_eq!((*module_def.ffi_def.get()).m_doc, DOC.as_ptr() as _);

            Python::attach(|py| {
                module_def.initializer.0(&py.import("builtins").unwrap()).unwrap();
                assert!(INIT_CALLED.load(Ordering::SeqCst));
            })
        }
    }
}
