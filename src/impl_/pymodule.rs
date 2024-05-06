//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::{cell::UnsafeCell, ffi::CStr, marker::PhantomData};

#[cfg(all(
    not(any(PyPy, GraalPy)),
    Py_3_9,
    not(all(windows, Py_LIMITED_API, not(Py_3_10))),
    not(target_has_atomic = "64"),
))]
use portable_atomic::{AtomicI64, Ordering};
#[cfg(all(
    not(any(PyPy, GraalPy)),
    Py_3_9,
    not(all(windows, Py_LIMITED_API, not(Py_3_10))),
    target_has_atomic = "64",
))]
use std::sync::atomic::{AtomicI64, Ordering};

#[cfg(not(any(PyPy, GraalPy)))]
use crate::exceptions::PyImportError;
use crate::{
    ffi,
    sync::GILOnceCell,
    types::{PyCFunction, PyModule, PyModuleMethods},
    Bound, Py, PyClass, PyMethodDef, PyResult, PyTypeInfo, Python,
};

use crate::impl_::pymodule_state as state;

// TODO: replace other usages (if this passes review :^) )
pub use state::ModuleDefSlot;

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
    /// Interpreter ID where module was initialized (not applicable on PyPy).
    #[cfg(all(
        not(any(PyPy, GraalPy)),
        Py_3_9,
        not(all(windows, Py_LIMITED_API, not(Py_3_10)))
    ))]
    interpreter: AtomicI64,
    // TODO: `module` could probably go..?
    /// Initialized module object, cached to avoid reinitialization.
    #[allow(unused)]
    module: GILOnceCell<Py<PyModule>>,
}

unsafe impl Sync for ModuleDef {}

impl ModuleDef {
    /// Make new module definition with given module name.
    pub const unsafe fn new(name: &'static CStr, doc: &'static CStr) -> Self {
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
            // -1 is never expected to be a valid interpreter ID
            #[cfg(all(
                not(any(PyPy, GraalPy)),
                Py_3_9,
                not(all(windows, Py_LIMITED_API, not(Py_3_10)))
            ))]
            interpreter: AtomicI64::new(-1),
            module: GILOnceCell::new(),
        }
    }

    /// Builds a module using user given initializer. Used for [`#[pymodule]`][crate::pymodule].
    pub fn make_module(&'static self, py: Python<'_>) -> PyResult<*mut ffi::PyModuleDef> {
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

        if (unsafe { *self.ffi_def.get() }).m_slots.is_null() {
            return Err(PyImportError::new_err(
                "'m_slots' of module definition is NULL",
            ));
        }

        let module_def_ptr = unsafe { ffi::PyModuleDef_Init(self.ffi_def.get()) };

        if module_def_ptr.is_null() {
            return Err(PyImportError::new_err("PyModuleDef_Init returned NULL"));
        }

        Ok(module_def_ptr.cast())
    }

    pub fn set_multiphase_items(&'static self, slots: state::ModuleDefSlots) {
        let ffi_def = self.ffi_def.get();
        unsafe {
            (*ffi_def).m_size = std::mem::size_of::<state::ModuleState>() as ffi::Py_ssize_t;
            (*ffi_def).m_slots = slots.into_inner();
            (*ffi_def).m_traverse = Some(state::module_state_traverse);
            (*ffi_def).m_clear = Some(state::module_state_clear);
            (*ffi_def).m_free = Some(state::module_state_free);
        };
    }
}

/// Trait to add an element (class, function...) to a module.
///
/// Currently only implemented for classes.
pub trait PyAddToModule {
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
        module.add(T::NAME, T::type_object_bound(module.py()))
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
        let parent_ptr = module.as_ptr();
        let parent_name = std::ffi::CString::new(module.name()?.to_string())?;

        let add_to_parent = |child_ptr: *mut ffi::PyObject| -> std::ffi::c_int {
            // TODO: reference to child_ptr is stolen - check if this is fine here?
            let ret =
                unsafe { ffi::PyModule_AddObject(parent_ptr, parent_name.as_ptr(), child_ptr) };

            // TODO: .. as well as this error handling here - is this fine
            // inside Py_mod_exec slots?
            if ret < 0 {
                unsafe { ffi::Py_DECREF(parent_ptr) };
                return -1;
            }

            0
        };

        // SAFETY: We only use this closure inside the ModuleDef's slots and
        // then immediately initialize the module - this closure /
        // "function pointer" isn't used anywhere else afterwards and can't
        // outlive the current thread.
        let add_to_parent = unsafe { state::alloc_closure(add_to_parent) };

        let slots = [
            state::ModuleDefSlot::start(),
            state::ModuleDefSlot::new(ffi::Py_mod_exec, add_to_parent),
            #[cfg(Py_3_12)]
            state::ModuleDefSlot::per_interpreter_gil(),
            state::ModuleDefSlot::end(),
        ];

        let slots = state::alloc_slots(slots);
        self.set_multiphase_items(slots);

        let _module_def_ptr = self.make_module(module.py())?;

        Ok(())
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
        impl_::pymodule_state as state,
        types::{any::PyAnyMethods, module::PyModuleMethods, PyModule},
        Bound, PyResult, Python,
    };

    use super::ModuleDef;

    #[test]
    fn module_init() {
        static MODULE_DEF: ModuleDef =
            unsafe { ModuleDef::new(ffi::c_str!("test_module"), ffi::c_str!("some doc")) };

        let slots = [
            state::ModuleDefSlot::start(),
            #[cfg(Py_3_12)]
            state::ModuleDefSlot::per_interpreter_gil(),
            state::ModuleDefSlot::end(),
        ];

        MODULE_DEF.set_multiphase_items(state::alloc_slots(slots));

        Python::with_gil(|py| {
            let module_def = MODULE_DEF.make_module(py).unwrap();
            // FIXME: get PyModule from PyModuleDef ..?
            unimplemented!("Test currently not implemented");
        })
    }

    #[test]
    fn module_def_new() {
        // To get coverage for ModuleDef::new() need to create a non-static ModuleDef, however init
        // etc require static ModuleDef, so this test needs to be separated out.
        static NAME: &CStr = ffi::c_str!("test_module");
        static DOC: &CStr = ffi::c_str!("some doc");

        let slots = [
            state::ModuleDefSlot::start(),
            #[cfg(Py_3_12)]
            state::ModuleDefSlot::per_interpreter_gil(),
            state::ModuleDefSlot::end(),
        ];

        static INIT_CALLED: AtomicBool = AtomicBool::new(false);

        #[allow(clippy::unnecessary_wraps)]
        fn init(_: &Bound<'_, PyModule>) -> PyResult<()> {
            INIT_CALLED.store(true, Ordering::SeqCst);
            Ok(())
        }

        unsafe {
            static MODULE_DEF: ModuleDef = unsafe { ModuleDef::new(NAME, DOC) };
            MODULE_DEF.set_multiphase_items(state::alloc_slots(slots));
            assert_eq!((*MODULE_DEF.ffi_def.get()).m_name, NAME.as_ptr() as _);
            assert_eq!((*MODULE_DEF.ffi_def.get()).m_doc, DOC.as_ptr() as _);

            Python::with_gil(|_py| {
                assert!(INIT_CALLED.load(Ordering::SeqCst));
            })
        }
    }
}
