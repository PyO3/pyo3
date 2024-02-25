//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::cell::UnsafeCell;

#[cfg(all(not(PyPy), Py_3_9, not(all(windows, Py_LIMITED_API, not(Py_3_10)))))]
use portable_atomic::{AtomicI64, Ordering};

#[cfg(not(PyPy))]
use crate::exceptions::PyImportError;
use crate::types::module::PyModuleMethods;
use crate::{ffi, sync::GILOnceCell, types::PyModule, Bound, Py, PyResult, PyTypeInfo, Python};

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
    initializer: ModuleInitializer,
    /// Interpreter ID where module was initialized (not applicable on PyPy).
    #[cfg(all(not(PyPy), Py_3_9, not(all(windows, Py_LIMITED_API, not(Py_3_10)))))]
    interpreter: AtomicI64,
    /// Initialized module object, cached to avoid reinitialization.
    module: GILOnceCell<Py<PyModule>>,
}

/// Wrapper to enable initializer to be used in const fns.
pub struct ModuleInitializer(pub for<'py> fn(&Bound<'py, PyModule>) -> PyResult<()>);

unsafe impl Sync for ModuleDef {}

impl ModuleDef {
    /// Make new module definition with given module name.
    ///
    /// # Safety
    /// `name` and `doc` must be null-terminated strings.
    pub const unsafe fn new(
        name: &'static str,
        doc: &'static str,
        initializer: ModuleInitializer,
    ) -> Self {
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
            m_name: name.as_ptr() as *const _,
            m_doc: doc.as_ptr() as *const _,
            ..INIT
        });

        ModuleDef {
            ffi_def,
            initializer,
            // -1 is never expected to be a valid interpreter ID
            #[cfg(all(not(PyPy), Py_3_9, not(all(windows, Py_LIMITED_API, not(Py_3_10)))))]
            interpreter: AtomicI64::new(-1),
            module: GILOnceCell::new(),
        }
    }
    /// Builds a module using user given initializer. Used for [`#[pymodule]`][crate::pymodule].
    pub fn make_module(&'static self, py: Python<'_>) -> PyResult<Py<PyModule>> {
        #[cfg(all(PyPy, not(Py_3_8)))]
        {
            use crate::types::any::PyAnyMethods;
            const PYPY_GOOD_VERSION: [u8; 3] = [7, 3, 8];
            let version = py
                .import_bound("sys")?
                .getattr("implementation")?
                .getattr("version")?;
            if version.lt(crate::types::PyTuple::new_bound(py, PYPY_GOOD_VERSION))? {
                let warn = py.import_bound("warnings")?.getattr("warn")?;
                warn.call1((
                    "PyPy 3.7 versions older than 7.3.8 are known to have binary \
                        compatibility issues which may cause segfaults. Please upgrade.",
                ))?;
            }
        }
        // Check the interpreter ID has not changed, since we currently have no way to guarantee
        // that static data is not reused across interpreters.
        //
        // PyPy does not have subinterpreters, so no need to check interpreter ID.
        #[cfg(not(PyPy))]
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
                self.initializer.0(module.bind(py))?;
                Ok(module)
            })
            .map(|py_module| py_module.clone_ref(py))
    }
}

/// Trait to add an element (class, function...) to a module.
///
/// Currently only implemented for classes.
pub trait PyAddToModule {
    fn add_to_module(module: &Bound<'_, PyModule>) -> PyResult<()>;
}

impl<T: PyTypeInfo> PyAddToModule for T {
    fn add_to_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add(Self::NAME, Self::type_object_bound(module.py()))
    }
}

pub struct BoundModule<'a, 'py>(pub &'a Bound<'py, PyModule>);

impl<'a> From<BoundModule<'a, 'a>> for &'a PyModule {
    #[inline]
    fn from(bound: BoundModule<'a, 'a>) -> Self {
        bound.0.as_gil_ref()
    }
}

impl<'a, 'py> From<BoundModule<'a, 'py>> for &'a Bound<'py, PyModule> {
    #[inline]
    fn from(bound: BoundModule<'a, 'py>) -> Self {
        bound.0
    }
}

impl From<BoundModule<'_, '_>> for Py<PyModule> {
    #[inline]
    fn from(bound: BoundModule<'_, '_>) -> Self {
        bound.0.clone().unbind()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use crate::{
        types::{any::PyAnyMethods, module::PyModuleMethods, PyModule},
        Bound, PyResult, Python,
    };

    use super::{ModuleDef, ModuleInitializer};

    #[test]
    fn module_init() {
        static MODULE_DEF: ModuleDef = unsafe {
            ModuleDef::new(
                "test_module\0",
                "some doc\0",
                ModuleInitializer(|m| {
                    m.add("SOME_CONSTANT", 42)?;
                    Ok(())
                }),
            )
        };
        Python::with_gil(|py| {
            let module = MODULE_DEF.make_module(py).unwrap().into_bound(py);
            assert_eq!(
                module
                    .getattr("__name__")
                    .unwrap()
                    .extract::<&str>()
                    .unwrap(),
                "test_module",
            );
            assert_eq!(
                module
                    .getattr("__doc__")
                    .unwrap()
                    .extract::<&str>()
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
        static NAME: &str = "test_module\0";
        static DOC: &str = "some doc\0";

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

            Python::with_gil(|py| {
                module_def.initializer.0(&py.import_bound("builtins").unwrap()).unwrap();
                assert!(INIT_CALLED.load(Ordering::SeqCst));
            })
        }
    }
}
