use core::ffi::{c_char, CStr};
use core::ptr;
use std::thread;

use pyo3_ffi::*;
use sequential::PyInit_sequential;

static COMMAND: &'static str = "
from sequential import Id

s = sum(int(Id()) for _ in range(12))
\0";

// Newtype to be able to pass it to another thread.
struct State(*mut PyThreadState);
unsafe impl Sync for State {}
unsafe impl Send for State {}

#[test]
fn lets_go_fast() -> Result<(), String> {
    unsafe {
        let ret = PyImport_AppendInittab(
            "sequential\0".as_ptr().cast::<c_char>(),
            Some(PyInit_sequential),
        );
        if ret == -1 {
            return Err("could not add module to inittab".into());
        }

        Py_Initialize();

        let main_state = PyThreadState_Swap(ptr::null_mut());

        const NULL: State = State(ptr::null_mut());
        let mut subs = [NULL; 12];

        let config = PyInterpreterConfig {
            use_main_obmalloc: 0,
            allow_fork: 0,
            allow_exec: 0,
            allow_threads: 1,
            allow_daemon_threads: 0,
            check_multi_interp_extensions: 1,
            gil: PyInterpreterConfig_OWN_GIL,
        };

        for State(state) in &mut subs {
            let status = Py_NewInterpreterFromConfig(state, &config);
            if PyStatus_IsError(status) == 1 {
                let msg = if status.err_msg.is_null() {
                    "no error message".into()
                } else {
                    CStr::from_ptr(status.err_msg).to_string_lossy()
                };
                PyThreadState_Swap(main_state);
                Py_FinalizeEx();
                return Err(format!("could not create new subinterpreter: {msg}"));
            }
        }

        PyThreadState_Swap(main_state);

        let main_state = PyEval_SaveThread(); // a PyInterpreterConfig with shared gil would deadlock otherwise

        let ints: Vec<_> = thread::scope(move |s| {
            let mut handles = vec![];

            for state in subs {
                let handle = s.spawn(move || {
                    let state = state;
                    PyEval_RestoreThread(state.0);

                    let ret = run_code();

                    Py_EndInterpreter(state.0);
                    ret
                });

                handles.push(handle);
            }

            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        PyEval_RestoreThread(main_state);

        let ret = Py_FinalizeEx();
        if ret == -1 {
            return Err("could not finalize interpreter".into());
        }

        let mut sum: u64 = 0;
        for i in ints {
            let i = i?;
            sum += i;
        }

        assert_eq!(sum, (0..).take(12 * 12).sum());
    }

    Ok(())
}

unsafe fn fetch() -> String {
    let err = PyErr_GetRaisedException();
    let err_repr = PyObject_Str(err);
    if !err_repr.is_null() {
        let mut size = 0;
        let p = PyUnicode_AsUTF8AndSize(err_repr, &mut size);
        if !p.is_null() {
            let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                p.cast::<u8>(),
                size as usize,
            ));
            let s = String::from(s);
            Py_DECREF(err_repr);
            return s;
        }
    }
    String::from("could not get error")
}

fn run_code() -> Result<u64, String> {
    unsafe {
        let code_obj = Py_CompileString(
            COMMAND.as_ptr().cast::<c_char>(),
            "program\0".as_ptr().cast::<c_char>(),
            Py_file_input,
        );
        if code_obj.is_null() {
            return Err(fetch());
        }
        let globals = PyDict_New();
        let res_ptr = PyEval_EvalCode(code_obj, globals, ptr::null_mut());
        Py_DECREF(code_obj);
        if res_ptr.is_null() {
            return Err(fetch());
        } else {
            Py_DECREF(res_ptr);
        }
        let sum = PyDict_GetItemString(globals, "s\0".as_ptr().cast::<c_char>()); /* borrowed reference */
        if sum.is_null() {
            Py_DECREF(globals);
            return Err("globals did not have `s`".into());
        }
        let int = PyLong_AsUnsignedLongLong(sum) as u64;

        Py_DECREF(globals);
        Ok(int)
    }
}
