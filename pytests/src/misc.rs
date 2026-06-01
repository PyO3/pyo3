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

// This exercises the py.detach() → reattach path during interpreter shutdown.
// The spawned thread attaches, releases the GIL via detach(), then blocks until the
// LockHolder is dropped during finalization. When the block returns, SuspendAttach::drop()
// calls PyEval_RestoreThread() on a finalizing interpreter.
#[pyfunction]
fn detach_then_reattach_in_thread(py: Python<'_>) -> LockHolder {
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        Python::attach(move |py| {
            py.detach(move || {
                // Signal that we're inside detach (GIL released) and blocked.
                ready_tx.send(()).ok();
                // Block until sender is dropped (during interpreter finalization).
                // When this returns, SuspendAttach::drop() calls PyEval_RestoreThread().
                receiver.recv().ok();
            });
        });
    });
    // Release the GIL while waiting to avoid deadlocking the spawned thread's
    // Python::attach() call, which needs the GIL to proceed.
    py.detach(move || {
        ready_rx.recv().ok();
    });
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
        accepts_bool, detach_then_reattach_in_thread, get_item_and_run_callback,
        get_type_fully_qualified_name, hammer_attaching_in_thread, issue_219,
    };
}
