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
    config._init_main = 0;

    // Set program_name as regression test for #2370
    #[cfg(all(Py_3_10, windows))]
    {
        use libc::wchar_t;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let mut value: Vec<wchar_t> = OsStr::new("some_test\0").encode_wide().collect();

        unsafe {
            ffi::PyConfig_SetString(&mut config, &mut config.program_name, value.as_ptr());
        }
    }
    #[cfg(all(Py_3_10, unix))]
    {
        unsafe {
            ffi::PyConfig_SetBytesString(
                &mut config,
                &mut config.program_name,
                "some_test\0".as_ptr().cast(),
            );
        }
    }

    ensure!(unsafe { ffi::Py_InitializeFromConfig(&config) });

    // The GIL is held.
    assert_eq!(unsafe { ffi::PyGILState_Check() }, 1);

    // Now proceed with the Python main initialization. This will initialize
    // importlib. And if the custom importlib bytecode was registered above,
    // our extension module will get imported and initialized.
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
