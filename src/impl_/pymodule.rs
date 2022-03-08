//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::cell::UnsafeCell;

use crate::{
    callback::panic_result_into_callback_output, ffi, types::PyModule, GILPool, IntoPyPointer, Py,
    PyObject, PyResult, Python,
};

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
    initializer: ModuleInitializer,
}

/// Wrapper to enable initializer to be used in const fns.
pub struct ModuleInitializer(pub fn(Python, &PyModule) -> PyResult<()>);

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
        }
    }
    /// Builds a module using user given initializer. Used for [`#[pymodule]`][crate::pymodule].
    pub fn make_module(&'static self, py: Python) -> PyResult<PyObject> {
        let module = unsafe {
            Py::<PyModule>::from_owned_ptr_or_err(py, ffi::PyModule_Create(self.ffi_def.get()))?
        };
        (self.initializer.0)(py, module.as_ref(py))?;
        Ok(module.into())
    }
    /// Implementation of `PyInit_foo` functions generated in [`#[pymodule]`][crate::pymodule]..
    ///
    /// # Safety
    /// The Python GIL must be held.
    pub unsafe fn module_init(&'static self) -> *mut ffi::PyObject {
        let pool = GILPool::new();
        let py = pool.python();
        let unwind_safe_self = std::panic::AssertUnwindSafe(self);
        panic_result_into_callback_output(
            py,
            std::panic::catch_unwind(move || -> PyResult<_> {
                #[cfg(all(PyPy, not(Py_3_8)))]
                {
                    const PYPY_GOOD_VERSION: [u8; 3] = [7, 3, 8];
                    let version = py
                        .import("sys")?
                        .getattr("implementation")?
                        .getattr("version")?;
                    if version.lt(crate::types::PyTuple::new(py, &PYPY_GOOD_VERSION))? {
                        let warn = py.import("warnings")?.getattr("warn")?;
                        warn.call1((
                            "PyPy 3.7 versions older than 7.3.8 are known to have binary \
                             compatibility issues which may cause segfaults. Please upgrade.",
                        ))?;
                    }
                }
                Ok(unwind_safe_self.make_module(py)?.into_ptr())
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use crate::{types::PyModule, PyResult, Python};

    use super::{ModuleDef, ModuleInitializer};

    #[test]
    fn module_init() {
        unsafe {
            static MODULE_DEF: ModuleDef = unsafe {
                ModuleDef::new(
                    "test_module\0",
                    "some doc\0",
                    ModuleInitializer(|_, m| {
                        m.add("SOME_CONSTANT", 42)?;
                        Ok(())
                    }),
                )
            };
            Python::with_gil(|py| {
                let module = MODULE_DEF.module_init();
                let pymodule: &PyModule = py.from_owned_ptr(module);
                assert_eq!(
                    pymodule
                        .getattr("__name__")
                        .unwrap()
                        .extract::<&str>()
                        .unwrap(),
                    "test_module",
                );
                assert_eq!(
                    pymodule
                        .getattr("__doc__")
                        .unwrap()
                        .extract::<&str>()
                        .unwrap(),
                    "some doc",
                );
                assert_eq!(
                    pymodule
                        .getattr("SOME_CONSTANT")
                        .unwrap()
                        .extract::<u8>()
                        .unwrap(),
                    42,
                );
            })
        }
    }

    #[test]
    fn module_def_new() {
        // To get coverage for ModuleDef::new() need to create a non-static ModuleDef, however init
        // etc require static ModuleDef, so this test needs to be separated out.
        static NAME: &str = "test_module\0";
        static DOC: &str = "some doc\0";

        static INIT_CALLED: AtomicBool = AtomicBool::new(false);

        #[allow(clippy::unnecessary_wraps)]
        fn init(_: Python, _: &PyModule) -> PyResult<()> {
            INIT_CALLED.store(true, Ordering::SeqCst);
            Ok(())
        }

        unsafe {
            let module_def: ModuleDef = ModuleDef::new(NAME, DOC, ModuleInitializer(init));
            assert_eq!((*module_def.ffi_def.get()).m_name, NAME.as_ptr() as _);
            assert_eq!((*module_def.ffi_def.get()).m_doc, DOC.as_ptr() as _);

            Python::with_gil(|py| {
                module_def.initializer.0(py, py.import("builtins").unwrap()).unwrap();
                assert!(INIT_CALLED.load(Ordering::SeqCst));
            })
        }
    }
}
