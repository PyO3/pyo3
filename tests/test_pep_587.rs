#![cfg(all(Py_3_8, not(any(PyPy, Py_LIMITED_API))))]

use pyo3::ffi;

#[cfg(Py_3_10)]
use widestring::WideCString;

#[test]
fn test_default_interpreter() {
    macro_rules! ensure {
        ($py_call:expr) => {{
            let status = $py_call;
            unsafe {
                if ffi::PyStatus_Exception(status) != 0 {
                    ffi::Py_ExitStatusException(status);
                }
            }
        }};
    }

    let mut preconfig = unsafe { std::mem::zeroed() };

    unsafe { ffi::PyPreConfig_InitPythonConfig(&mut preconfig) };
    preconfig.utf8_mode = 1;

    ensure!(unsafe { ffi::Py_PreInitialize(&preconfig) });

    let mut config = unsafe { std::mem::zeroed() };
    unsafe { ffi::PyConfig_InitPythonConfig(&mut config) };

    // Require manually calling _Py_InitializeMain to exercise more ffi code
    #[allow(clippy::used_underscore_binding)]
    {
        config._init_main = 0;
    }

    #[cfg(Py_3_10)]
    unsafe {
        ffi::PyConfig_SetBytesString(
            &mut config,
            &mut config.program_name,
            "some_test\0".as_ptr().cast(),
        );
    }

    ensure!(unsafe { ffi::Py_InitializeFromConfig(&config) });

    // The GIL is held.
    assert_eq!(unsafe { ffi::PyGILState_Check() }, 1);

    // Now proceed with the Python main initialization.
    ensure!(unsafe { ffi::_Py_InitializeMain() });

    // The GIL is held after finishing initialization.
    assert_eq!(unsafe { ffi::PyGILState_Check() }, 1);

    // Confirm program name set above was picked up correctly
    #[cfg(Py_3_10)]
    {
        let program_name = unsafe { WideCString::from_ptr_str(ffi::Py_GetProgramName().cast()) };
        assert_eq!(program_name.to_string().unwrap(), "some_test");
    }
}
