use std::cell::Cell;

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

#[pyclass]
struct MustDropWhileAttached;

impl Drop for MustDropWhileAttached {
    fn drop(&mut self) {
        // SAFETY: always callable; fatal error (abort) if the thread is not attached.
        unsafe { pyo3::ffi::PyThreadState_Get() };
    }
}

thread_local! {
    // Dropped when the thread exits, which on older CPython happens inside
    // PyEval_RestoreThread when reattaching during finalization.
    static DROPPED_ON_THREAD_EXIT: Cell<Option<Py<MustDropWhileAttached>>> = const { Cell::new(None) };
}

#[pyfunction]
fn detach_during_finalization(py: Python<'_>) -> LockHolder {
    let (sender, receiver) = std::sync::mpsc::channel();
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        Python::attach(|py| {
            DROPPED_ON_THREAD_EXIT.set(Some(Py::new(py, MustDropWhileAttached).unwrap()));
            ready_sender.send(()).unwrap();
            py.detach(move || receiver.recv().ok());
            // Interpreter is finalizing while we try to reattach after returning
        });
    });
    py.detach(move || ready_receiver.recv()).unwrap();
    LockHolder { sender }
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
        accepts_bool, detach_during_finalization, get_item_and_run_callback,
        get_type_fully_qualified_name, hammer_attaching_in_thread, issue_219,
    };
}
