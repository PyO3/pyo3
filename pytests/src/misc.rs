use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

#[pyfunction]
fn issue_219() {
    // issue 219: attaching inside #[pyfunction] deadlocks.
    Python::attach(|_| {});
}

#[pyclass]
struct LockHolder {
    #[expect(unused, reason = "used to block until sender is dropped")]
    sender: std::sync::mpsc::Sender<()>,
}

// This will repeatedly attach and detach from the Python interpreter
// once the LockHolder is dropped.
#[pyfunction]
fn hammer_attaching_in_thread() -> LockHolder {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        receiver.recv().ok();
        // now the interpreter has shut down, so hammer the attach API. In buggy
        // versions of PyO3 this will cause a crash.
        loop {
            Python::try_attach(|_py| ());
        }
    });
    LockHolder { sender }
}

// Returns true once the interpreter has begun finalizing.
// _Py_IsFinalizing() is the private exported function on Python < 3.13.
// Py_IsFinalizing() is public and exported on Python 3.13+ (where _Py_IsFinalizing was removed).
// Both are safe to call without the GIL — they read an atomic variable.
// PyO3 correctly only exports it on 3.13+ but for this unit test we can access the
// older private version directly.
fn is_finalizing() -> bool {
    #[cfg(not(Py_3_13))]
    {
        extern "C" {
            fn _Py_IsFinalizing() -> std::ffi::c_int;
        }
        unsafe { _Py_IsFinalizing() != 0 }
    }
    #[cfg(Py_3_13)]
    {
        unsafe { pyo3::ffi::Py_IsFinalizing() != 0 }
    }
}

// Called from a Python daemon thread. Releases the GIL and polls Py_IsFinalizing().
// When finalization begins, exits the closure, triggering SuspendAttach::drop()
// → PyEval_RestoreThread() on a finalizing interpreter.
#[pyfunction]
fn block_in_detach_until_finalizing(py: Python<'_>) {
    py.detach(|| {
        while !is_finalizing() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
}

#[pyfunction]
fn get_type_fully_qualified_name<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyString>> {
    obj.get_type().fully_qualified_name()
}

#[pyfunction]
fn accepts_bool(val: bool) -> bool {
    val
}

#[pyfunction]
fn get_item_and_run_callback(dict: Bound<'_, PyDict>, callback: Bound<'_, PyAny>) -> PyResult<()> {
    // This function gives the opportunity to run a pure-Python callback so that
    // gevent can instigate a context switch. This had problematic interactions
    // with PyO3's removed "GIL Pool".
    // For context, see https://github.com/PyO3/pyo3/issues/3668
    let item = dict.get_item("key")?.expect("key not found in dict");
    let string = item.to_string();
    callback.call0()?;
    assert_eq!(item.to_string(), string);
    Ok(())
}

#[pymodule]
pub mod misc {
    #[pymodule_export]
    use super::{
        accepts_bool, block_in_detach_until_finalizing, get_item_and_run_callback,
        get_type_fully_qualified_name, hammer_attaching_in_thread, issue_219,
    };
}
