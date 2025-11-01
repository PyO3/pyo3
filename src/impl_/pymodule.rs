//! Implementation details of `#[pymodule]` which need to be accessible from proc-macro generated code.

use std::{
    cell::UnsafeCell,
    ffi::CStr,
    marker::PhantomData,
    os::raw::{c_int, c_void},
};

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
use std::sync::atomic::{AtomicI64, Ordering};

#[cfg(not(any(PyPy, GraalPy)))]
use crate::exceptions::PyImportError;
use crate::prelude::PyTypeMethods;
use crate::{
    ffi,
    impl_::pyfunction::PyFunctionDef,
    sync::PyOnceLock,
    types::{any::PyAnyMethods, dict::PyDictMethods, PyDict, PyModule, PyModuleMethods},
    Bound, Py, PyAny, PyClass, PyResult, PyTypeInfo, Python,
};
use crate::{ffi_ptr_ext::FfiPtrExt, PyErr};

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
    /// Initialized module object, cached to avoid reinitialization.
    module: PyOnceLock<Py<PyModule>>,
}

unsafe impl Sync for ModuleDef {}

impl ModuleDef {
    /// Make new module definition with given module name.
    pub const fn new<const N: usize>(
        name: &'static CStr,
        doc: &'static CStr,
        // TODO: it might be nice to make this unsized and not need the
        // const N generic parameter, however that might need unsized return values
        // or other messy hacks.
        slots: &'static PyModuleSlots<N>,
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
            // TODO: would be slightly nicer to use `[T]::as_mut_ptr()` here,
            // but that requires mut ptr deref on MSRV.
            m_slots: slots.0.get() as _,
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
            module: PyOnceLock::new(),
        }
    }

    pub fn init_multi_phase(&'static self) -> *mut ffi::PyObject {
        // SAFETY: `ffi_def` is correctly initialized in `new()`
        unsafe { ffi::PyModuleDef_Init(self.ffi_def.get()) }
    }

    /// Builds a module object directly. Used for [`#[pymodule]`][crate::pymodule] submodules.
    pub fn make_module(&'static self, py: Python<'_>) -> PyResult<Py<PyModule>> {
        // Check the interpreter ID has not changed, since we currently have no way to guarantee
        // that static data is not reused across interpreters.
        //
        // PyPy does not have subinterpreters, so no need to check interpreter ID.
        //
        // TODO: it should be possible to use the Py_mod_multiple_interpreters slot on sufficiently
        // new Python versions to remove the need for this custom logic
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

        // Make a dummy spec, needs a `name` attribute and that seems to be sufficient
        // for the loader system

        static SIMPLE_NAMESPACE: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
        let simple_ns = SIMPLE_NAMESPACE.import(py, "types", "SimpleNamespace")?;

        let ffi_def = self.ffi_def.get();

        let name = unsafe { CStr::from_ptr((*ffi_def).m_name).to_str()? }.to_string();
        let kwargs = PyDict::new(py);
        kwargs.set_item("name", name)?;
        let spec = simple_ns.call((), Some(&kwargs))?;

        self.module
            .get_or_try_init(py, || {
                let def = self.ffi_def.get();
                let module = unsafe {
                    ffi::PyModule_FromDefAndSpec(def, spec.as_ptr()).assume_owned_or_err(py)?
                }
                .cast_into()?;
                if unsafe { ffi::PyModule_ExecDef(module.as_ptr(), def) } != 0 {
                    return Err(PyErr::fetch(py));
                }
                Ok(module.unbind())
            })
            .map(|py_module| py_module.clone_ref(py))
    }
}

/// Type of the exec slot used to initialise module contents
pub type ModuleExecSlot = unsafe extern "C" fn(*mut ffi::PyObject) -> c_int;

/// Builder to create `PyModuleSlots`. The size of the number of slots desired must
/// be known up front, and N needs to be at least one greater than the number of
/// actual slots pushed due to the need to have a zeroed element on the end.
pub struct PyModuleSlotsBuilder<const N: usize> {
    // values (initially all zeroed)
    values: [ffi::PyModuleDef_Slot; N],
    // current length
    len: usize,
}

impl<const N: usize> PyModuleSlotsBuilder<N> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            values: [unsafe { std::mem::zeroed() }; N],
            len: 0,
        }
    }

    pub const fn with_mod_exec(self, exec: ModuleExecSlot) -> Self {
        self.push(ffi::Py_mod_exec, exec as *mut c_void)
    }

    pub const fn with_gil_used(self, gil_used: bool) -> Self {
        #[cfg(Py_3_13)]
        {
            self.push(
                ffi::Py_mod_gil,
                if gil_used {
                    ffi::Py_MOD_GIL_USED
                } else {
                    ffi::Py_MOD_GIL_NOT_USED
                },
            )
        }

        #[cfg(not(Py_3_13))]
        {
            // Silence unused variable warning
            let _ = gil_used;

            // Py_mod_gil didn't exist before 3.13, can just make
            // this function a noop.
            //
            // By handling it here we can avoid conditional
            // compilation within the macros; they can always emit
            // a `.with_gil_used()` call.
            self
        }
    }

    pub const fn build(self) -> PyModuleSlots<N> {
        // Required to guarantee there's still a zeroed element
        // at the end
        assert!(
            self.len < N,
            "N must be greater than the number of slots pushed"
        );
        PyModuleSlots(UnsafeCell::new(self.values))
    }

    const fn push(mut self, slot: c_int, value: *mut c_void) -> Self {
        self.values[self.len] = ffi::PyModuleDef_Slot { slot, value };
        self.len += 1;
        self
    }
}

/// Wrapper to safely store module slots, to be used in a `ModuleDef`.
pub struct PyModuleSlots<const N: usize>(UnsafeCell<[ffi::PyModuleDef_Slot; N]>);

// It might be possible to avoid this with SyncUnsafeCell in the future
//
// SAFETY: the inner values are only accessed within a `ModuleDef`,
// which only uses them to build the `ffi::ModuleDef`.
unsafe impl<const N: usize> Sync for PyModuleSlots<N> {}

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
        let object = T::type_object(module.py());
        module.add(object.name()?, object)
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
impl PyAddToModule for PyFunctionDef {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        // safety: self is static
        module.add_function(self.create_py_c_function(module.py(), Some(module))?)
    }
}

/// For adding a module to a module.
impl PyAddToModule for ModuleDef {
    fn add_to_module(&'static self, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_submodule(self.make_module(module.py())?.bind(module.py()))
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, ffi::CStr, os::raw::c_int};

    use crate::{
        ffi,
        impl_::{
            pymodule::{PyModuleSlots, PyModuleSlotsBuilder},
            trampoline,
        },
        types::{any::PyAnyMethods, module::PyModuleMethods},
        Python,
    };

    use super::ModuleDef;

    #[test]
    fn module_init() {
        unsafe extern "C" fn module_exec(module: *mut ffi::PyObject) -> c_int {
            unsafe {
                trampoline::module_exec(module, |m| {
                    m.add("SOME_CONSTANT", 42)?;
                    Ok(())
                })
            }
        }

        static SLOTS: PyModuleSlots<2> = PyModuleSlotsBuilder::new()
            .with_mod_exec(module_exec)
            .build();
        static MODULE_DEF: ModuleDef = ModuleDef::new(c"test_module", c"some doc", &SLOTS);

        Python::attach(|py| {
            let module = MODULE_DEF.make_module(py).unwrap().into_bound(py);
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
        static NAME: &CStr = c"test_module";
        static DOC: &CStr = c"some doc";

        static SLOTS: PyModuleSlots<2> = PyModuleSlotsBuilder::new().build();

        unsafe {
            let module_def: ModuleDef = ModuleDef::new(NAME, DOC, &SLOTS);
            assert_eq!((*module_def.ffi_def.get()).m_name, NAME.as_ptr() as _);
            assert_eq!((*module_def.ffi_def.get()).m_doc, DOC.as_ptr() as _);
            assert_eq!((*module_def.ffi_def.get()).m_slots, SLOTS.0.get().cast());
        }
    }

    #[test]
    #[should_panic]
    fn test_module_slots_builder_overflow() {
        unsafe extern "C" fn module_exec(_module: *mut ffi::PyObject) -> c_int {
            0
        }

        PyModuleSlotsBuilder::<0>::new().with_mod_exec(module_exec);
    }

    #[test]
    #[should_panic]
    fn test_module_slots_builder_overflow_2() {
        unsafe extern "C" fn module_exec(_module: *mut ffi::PyObject) -> c_int {
            0
        }

        PyModuleSlotsBuilder::<2>::new()
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .build();
    }
}
