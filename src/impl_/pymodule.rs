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
))]
use std::sync::atomic::Ordering;

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

#[cfg(not(any(PyPy, GraalPy)))]
use crate::exceptions::PyImportError;
use crate::prelude::PyTypeMethods;
use crate::{
    ffi,
    impl_::pyfunction::PyFunctionDef,
    types::{PyModule, PyModuleMethods},
    Bound, PyClass, PyResult, PyTypeInfo,
};
use crate::{ffi_ptr_ext::FfiPtrExt, PyErr};
use crate::{
    sync::PyOnceLock,
    types::{any::PyAnyMethods, dict::PyDictMethods, PyDict},
    Py, PyAny, Python,
};

/// `Sync` wrapper of `ffi::PyModuleDef`.
pub struct ModuleDef {
    // wrapped in UnsafeCell so that Rust compiler treats this as interior mutability
    #[cfg(not(_Py_OPAQUE_PYOBJECT))]
    ffi_def: UnsafeCell<ffi::PyModuleDef>,
    #[cfg(Py_3_15)]
    name: &'static CStr,
    #[cfg(Py_3_15)]
    doc: &'static CStr,
    slots: &'static PyModuleSlots,
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
    pub const fn new(
        name: &'static CStr,
        doc: &'static CStr,
        slots: &'static PyModuleSlots,
    ) -> Self {
        // This is only used in PyO3 for append_to_inittab on Python 3.15 and newer.
        // There could also be other tools that need the legacy init hook.
        // Opaque PyObject builds won't be able to use this.
        #[cfg(not(_Py_OPAQUE_PYOBJECT))]
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

        #[cfg(not(_Py_OPAQUE_PYOBJECT))]
        let ffi_def = UnsafeCell::new(ffi::PyModuleDef {
            m_name: name.as_ptr(),
            m_doc: doc.as_ptr(),
            // TODO: would be slightly nicer to use `[T]::as_mut_ptr()` here,
            // but that requires mut ptr deref on MSRV.
            m_slots: slots.0.get() as _,
            ..INIT
        });

        ModuleDef {
            #[cfg(not(_Py_OPAQUE_PYOBJECT))]
            ffi_def,
            #[cfg(Py_3_15)]
            name,
            #[cfg(Py_3_15)]
            doc,
            slots,
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
        #[cfg(not(_Py_OPAQUE_PYOBJECT))]
        unsafe {
            ffi::PyModuleDef_Init(self.ffi_def.get())
        }
        #[cfg(_Py_OPAQUE_PYOBJECT)]
        panic!("TODO: fix this panic");
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

        #[cfg(not(Py_3_15))]
        {
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

        #[cfg(Py_3_15)]
        {
            let name = self.name;
            let doc = self.doc;
            let kwargs = PyDict::new(py);
            kwargs.set_item("name", name)?;
            let spec = simple_ns.call((), Some(&kwargs))?;

            self.module
                .get_or_try_init(py, || {
                    let slots = self.get_slots();
                    let module = unsafe { ffi::PyModule_FromSlotsAndSpec(slots, spec.as_ptr()) };
                    if unsafe { ffi::PyModule_SetDocString(module, doc.as_ptr()) } != 0 {
                        return Err(PyErr::fetch(py));
                    }
                    let module = unsafe { module.assume_owned_or_err(py)? }.cast_into()?;
                    if unsafe { ffi::PyModule_Exec(module.as_ptr()) } != 0 {
                        return Err(PyErr::fetch(py));
                    }
                    Ok(module.unbind())
                })
                .map(|py_module| py_module.clone_ref(py))
        }
    }
    pub fn get_slots(&'static self) -> *mut ffi::PyModuleDef_Slot {
        self.slots.0.get() as *mut ffi::PyModuleDef_Slot
    }
}

/// Type of the exec slot used to initialise module contents
pub type ModuleExecSlot = unsafe extern "C" fn(*mut ffi::PyObject) -> c_int;

const MAX_SLOTS: usize =
    // Py_mod_exec and a trailing null entry
    2 +
    // Py_mod_gil
    cfg!(Py_3_13) as usize +
    // Py_mod_name, Py_mod_doc, and Py_mod_abi
    3 * (cfg!(Py_3_15) as usize);

/// Builder to create `PyModuleSlots`. The size of the number of slots desired must
/// be known up front, and N needs to be at least one greater than the number of
/// actual slots pushed due to the need to have a zeroed element on the end.
pub struct PyModuleSlotsBuilder {
    // values (initially all zeroed)
    values: [ffi::PyModuleDef_Slot; MAX_SLOTS],
    // current length
    len: usize,
}

// note that macros cannot use conditional compilation,
// so all implementations below must be available in all
// Python versions
// By handling it here we can avoid conditional
// compilation within the macros; they can always emit
// e.g. a `.with_gil_used()` call.
impl PyModuleSlotsBuilder {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            values: [unsafe { std::mem::zeroed() }; MAX_SLOTS],
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
            self
        }
    }

    pub const fn with_name(self, name: &'static CStr) -> Self {
        #[cfg(Py_3_15)]
        {
            self.push(ffi::Py_mod_name, name.as_ptr() as *mut c_void)
        }

        #[cfg(not(Py_3_15))]
        {
            // Silence unused variable warning
            let _ = name;
            self
        }
    }

    pub const fn with_abi_info(self) -> Self {
        #[cfg(Py_3_15)]
        {
            ffi::PyABIInfo_VAR!(ABI_INFO);
            self.push(ffi::Py_mod_abi, std::ptr::addr_of_mut!(ABI_INFO).cast())
        }

        #[cfg(not(Py_3_15))]
        {
            self
        }
    }

    pub const fn with_doc(self, doc: &'static CStr) -> Self {
        #[cfg(Py_3_15)]
        {
            self.push(ffi::Py_mod_doc, doc.as_ptr() as *mut c_void)
        }

        #[cfg(not(Py_3_15))]
        {
            // Silence unused variable warning
            let _ = doc;
            self
        }
    }

    pub const fn build(self) -> PyModuleSlots {
        // Required to guarantee there's still a zeroed element
        // at the end
        assert!(
            self.len < MAX_SLOTS,
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
pub struct PyModuleSlots(UnsafeCell<[ffi::PyModuleDef_Slot; MAX_SLOTS]>);

// It might be possible to avoid this with SyncUnsafeCell in the future
//
// SAFETY: the inner values are only accessed within a `ModuleDef`,
// which only uses them to build the `ffi::ModuleDef`.
unsafe impl Sync for PyModuleSlots {}

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

        static NAME: &CStr = c"test_module";
        static DOC: &CStr = c"some doc";

        static SLOTS: PyModuleSlots = PyModuleSlotsBuilder::new()
            .with_mod_exec(module_exec)
            .with_gil_used(false)
            .with_abi_info()
            .with_name(NAME)
            .with_doc(DOC)
            .build();

        static MODULE_DEF: ModuleDef = ModuleDef::new(NAME, DOC, &SLOTS);

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

        static SLOTS: PyModuleSlots = PyModuleSlotsBuilder::new().build();

        let module_def: ModuleDef = ModuleDef::new(NAME, DOC, &SLOTS);

        #[cfg(not(_Py_OPAQUE_PYOBJECT))]
        unsafe {
            assert_eq!((*module_def.ffi_def.get()).m_slots, SLOTS.0.get().cast());
        }
        #[cfg(Py_3_15)]
        assert_eq!(module_def.name, NAME);
        #[cfg(Py_3_15)]
        assert_eq!(module_def.doc, DOC);
        assert_eq!(module_def.slots.0.get(), SLOTS.0.get());
    }

    #[test]
    #[should_panic]
    fn test_module_slots_builder_overflow_2() {
        unsafe extern "C" fn module_exec(_module: *mut ffi::PyObject) -> c_int {
            0
        }

        PyModuleSlotsBuilder::new()
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .with_mod_exec(module_exec)
            .build();
    }
}
