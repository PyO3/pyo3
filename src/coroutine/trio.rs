//! Coroutine implementation compatible with trio.
use pyo3_macros::pyfunction;

use crate::{
    intern,
    sync::GILOnceCell,
    types::{PyAnyMethods, PyCFunction, PyIterator},
    wrap_pyfunction_bound, Bound, Py, PyAny, PyObject, PyResult, Python,
};

struct Trio {
    cancel_shielded_checkpoint: PyObject,
    current_task: PyObject,
    current_trio_token: PyObject,
    reschedule: PyObject,
    succeeded: PyObject,
    wait_task_rescheduled: PyObject,
}
impl Trio {
    fn get(py: Python<'_>) -> PyResult<&Self> {
        static TRIO: GILOnceCell<Trio> = GILOnceCell::new();
        TRIO.get_or_try_init(py, || {
            let module = py.import_bound("trio.lowlevel")?;
            Ok(Self {
                cancel_shielded_checkpoint: module.getattr("cancel_shielded_checkpoint")?.into(),
                current_task: module.getattr("current_task")?.into(),
                current_trio_token: module.getattr("current_trio_token")?.into(),
                reschedule: module.getattr("reschedule")?.into(),
                succeeded: module.getattr("Abort")?.getattr("SUCCEEDED")?.into(),
                wait_task_rescheduled: module.getattr("wait_task_rescheduled")?.into(),
            })
        })
    }
}

fn yield_from(coro_func: &PyAny) -> PyResult<PyObject> {
    PyIterator::from_object(coro_func.call_method0("__await__")?)?
        .next()
        .expect("cancel_shielded_checkpoint didn't yield")
        .map(Into::into)
}

/// Asyncio-compatible coroutine waker.
///
/// Polling a Rust future yields `trio.lowlevel.wait_task_rescheduled()`, while `Waker::wake`
/// reschedule the current task.
pub(super) struct TrioWaker {
    task: PyObject,
    token: PyObject,
}

impl TrioWaker {
    pub(super) fn new(py: Python<'_>) -> PyResult<Self> {
        let trio = Trio::get(py)?;
        let task = trio.current_task.call0(py)?;
        let token = trio.current_trio_token.call0(py)?;
        Ok(Self { task, token })
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        static ABORT_FUNC: GILOnceCell<Py<PyCFunction>> = GILOnceCell::new();
        let abort_func = ABORT_FUNC.get_or_try_init(py, || {
            wrap_pyfunction_bound!(abort_func, py).map(Into::into)
        })?;
        let wait_task_rescheduled = Trio::get(py)?
            .wait_task_rescheduled
            .call1(py, (abort_func,))?;
        yield_from(wait_task_rescheduled.as_ref(py))
    }

    pub(super) fn yield_waken(py: Python<'_>) -> PyResult<PyObject> {
        let checkpoint = Trio::get(py)?.cancel_shielded_checkpoint.call0(py)?;
        yield_from(checkpoint.as_ref(py))
    }

    pub(super) fn wake(&self, py: Python<'_>) -> PyResult<()> {
        self.token.call_method1(
            py,
            intern!(py, "run_sync_soon"),
            (&Trio::get(py)?.reschedule, &self.task),
        )?;
        Ok(())
    }
}

#[pyfunction(crate = "crate")]
fn abort_func(py: Python<'_>, _arg: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    Ok(Trio::get(py)?.succeeded.clone())
}
