//! TODO

use std::{ffi::CString, mem::forget};

use crate::{
    conversion::PyTryInto,
    exceptions::PyTypeError,
    ffi,
    sync::GILOnceCell,
    type_object::PyTypeInfo,
    types::{PyCapsule, PyDict, PyModule},
    Py, PyResult, Python,
};

#[repr(C)]
pub(crate) struct GlobalApi {
    version: u64,
    pub(crate) create_panic_exception:
        unsafe extern "C" fn(msg_ptr: *const u8, msg_len: usize) -> *mut ffi::PyObject,
}

pub(crate) fn ensure_global_api(py: Python<'_>) -> PyResult<&GlobalApi> {
    let api = GLOBAL_API.0.get_or_try_init(py, || init_global_api(py))?;

    // SAFETY: We inserted the capsule if it was missing
    // and verified that it contains a compatible version.
    Ok(unsafe { &**api })
}

struct GlobalApiPtr(GILOnceCell<*const GlobalApi>);

unsafe impl Send for GlobalApiPtr {}

unsafe impl Sync for GlobalApiPtr {}

static GLOBAL_API: GlobalApiPtr = GlobalApiPtr(GILOnceCell::new());

#[cold]
fn init_global_api(py: Python<'_>) -> PyResult<*const GlobalApi> {
    let module = match PyModule::import(py, "pyo3") {
        Ok(module) => module,
        Err(_err) => {
            let module = PyModule::new(py, "pyo3")?;

            module.add(
                "PanicException",
                crate::panic::PanicException::type_object(py),
            )?;

            let sys = PyModule::import(py, "sys")?;
            let modules: &PyDict = sys.getattr("modules")?.downcast()?;
            modules.set_item("pyo3", module)?;

            module
        }
    };

    let capsule: &PyCapsule = match module.getattr("_GLOBAL_API") {
        Ok(capsule) => PyTryInto::try_into(capsule)?,
        Err(_err) => {
            let api = GlobalApi {
                version: 1,
                create_panic_exception: crate::panic::create_panic_exception,
            };

            let capsule = PyCapsule::new(py, api, Some(CString::new("_GLOBAL_API").unwrap()))?;
            module.setattr("_GLOBAL_API", capsule)?;
            capsule
        }
    };

    // SAFETY: All versions of the global API start with a version field.
    let version = unsafe { *(capsule.pointer() as *mut u64) };
    if version < 1 {
        return Err(PyTypeError::new_err(format!(
            "Version {} of global API is not supported by this version of PyO3",
            version
        )));
    }

    // Intentionally leak a reference to the capsule so we can safely cache a pointer into its interior.
    forget(Py::<PyCapsule>::from(capsule));

    Ok(capsule.pointer() as *const GlobalApi)
}
