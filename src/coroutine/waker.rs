use std::{
    cell::Cell,
    sync::Arc,
    task::{Poll, Wake, Waker},
};

use crate::{
    coroutine::{
        awaitable::{delegate, YieldOrReturn},
        CoroOp,
    },
    exceptions::PyStopIteration,
    intern,
    sync::GILOnceCell,
    types::PyAnyMethods,
    Bound, PyObject, PyResult, Python,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "anyio")] {
        type WakerImpl = crate::coroutine::anyio::AnyioWaker;
    } else {
        type WakerImpl = crate::coroutine::asyncio::AsyncioWaker;
    }
}

const MIXED_AWAITABLE_AND_FUTURE_ERROR: &str = "Python awaitable mixed with Rust future";

enum State {
    Pending(WakerImpl),
    Waken,
    Delegated(PyObject),
}

pub(super) struct CoroutineWaker {
    state: GILOnceCell<State>,
    op: CoroOp,
}

impl CoroutineWaker {
    pub(super) fn new(op: CoroOp) -> Self {
        Self {
            state: GILOnceCell::new(),
            op,
        }
    }

    pub(super) fn reset(&mut self, op: CoroOp) {
        self.state.take();
        self.op = op;
    }

    pub(super) fn is_delegated(&self, py: Python<'_>) -> bool {
        matches!(self.state.get(py), Some(State::Delegated(_)))
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        let init = || PyResult::Ok(State::Pending(WakerImpl::new(py)?));
        let state = self.state.get_or_try_init(py, init)?;
        match state {
            State::Pending(waker) => waker.yield_(py),
            State::Waken => WakerImpl::yield_waken(py),
            State::Delegated(obj) => Ok(obj.clone_ref(py)),
        }
    }

    fn delegate(&self, py: Python<'_>, await_impl: PyObject) -> Poll<PyResult<PyObject>> {
        match delegate(py, await_impl, &self.op) {
            Ok(YieldOrReturn::Yield(obj)) => {
                let delegated = self.state.set(py, State::Delegated(obj));
                assert!(delegated.is_ok(), "{}", MIXED_AWAITABLE_AND_FUTURE_ERROR);
                Poll::Pending
            }
            Ok(YieldOrReturn::Return(obj)) => Poll::Ready(Ok(obj)),
            Err(err) if err.is_instance_of::<PyStopIteration>(py) => Poll::Ready(
                err.value_bound(py)
                    .getattr(intern!(py, "value"))
                    .map(Bound::unbind),
            ),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl Wake for CoroutineWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        Python::with_gil(|gil| match WAKER_HACK.with(|cell| cell.take()) {
            Some(WakerHack::Argument(await_impl)) => WAKER_HACK.with(|cell| {
                let res = self.delegate(gil, await_impl);
                cell.set(Some(WakerHack::Result(res)))
            }),
            Some(WakerHack::Result(_)) => unreachable!(),
            None => match self.state.get_or_init(gil, || State::Waken) {
                State::Pending(waker) => waker.wake(gil).expect("wake error"),
                State::Waken => {}
                State::Delegated(_) => panic!("{}", MIXED_AWAITABLE_AND_FUTURE_ERROR),
            },
        })
    }
}

enum WakerHack {
    Argument(PyObject),
    Result(Poll<PyResult<PyObject>>),
}

thread_local! {
    static WAKER_HACK: Cell<Option<WakerHack>> = Cell::new(None);
}

pub(crate) fn try_delegate(
    waker: &Waker,
    await_impl: PyObject,
) -> Option<Poll<PyResult<PyObject>>> {
    WAKER_HACK.with(|cell| cell.set(Some(WakerHack::Argument(await_impl))));
    waker.wake_by_ref();
    match WAKER_HACK.with(|cell| cell.take()) {
        Some(WakerHack::Result(poll)) => Some(poll),
        Some(WakerHack::Argument(_)) => None,
        None => unreachable!(),
    }
}
