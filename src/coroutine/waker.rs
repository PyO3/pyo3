use std::{
    cell::Cell,
    sync::Arc,
    task::{Poll, Wake},
};

use crate::{
    coroutine::asyncio::AsyncioWaker, exceptions::PyStopIteration, intern, pyclass::IterNextOutput,
    sync::GILOnceCell, types::PyFuture, Py, PyNativeType, PyObject, PyResult, Python,
};

const MIXED_AWAITABLE_AND_FUTURE_ERROR: &str = "Python awaitable mixed with Rust future";

pub(crate) enum FutureOrPoll {
    Future(Py<PyFuture>),
    Poll(Poll<PyResult<PyObject>>),
}

thread_local! {
    pub(crate) static FUTURE_OR_POLL: Cell<Option<FutureOrPoll>> = Cell::new(None);
}

enum State {
    Pending(AsyncioWaker),
    Waken,
    Delegated(PyObject),
}

pub(super) struct CoroutineWaker {
    state: GILOnceCell<State>,
    sent_result: Option<Result<PyObject, PyObject>>,
}

impl CoroutineWaker {
    pub(super) fn new(sent_result: Option<Result<PyObject, PyObject>>) -> Self {
        Self {
            state: GILOnceCell::new(),
            sent_result,
        }
    }

    pub(super) fn reset(&mut self, sent_result: Option<Result<PyObject, PyObject>>) {
        self.state.take();
        self.sent_result = sent_result;
    }

    pub(super) fn is_delegated(&self, py: Python<'_>) -> bool {
        matches!(self.state.get(py), Some(State::Delegated(_)))
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        let init = || PyResult::Ok(State::Pending(AsyncioWaker::new(py)?));
        let state = self.state.get_or_try_init(py, init)?;
        match state {
            State::Waken => AsyncioWaker::yield_waken(py),
            State::Delegated(obj) => Ok(obj.clone_ref(py)),
            State::Pending(waker) => waker.yield_(py),
        }
    }

    fn delegate(&self, future: &PyFuture) -> Poll<PyResult<PyObject>> {
        let py = future.py();
        match future.next(&self.sent_result) {
            Ok(IterNextOutput::Return(ret)) => Poll::Ready(Ok(ret)),
            Ok(IterNextOutput::Yield(yielded)) => {
                let delegated = self.state.set(py, State::Delegated(yielded));
                assert!(delegated.is_ok(), "{}", MIXED_AWAITABLE_AND_FUTURE_ERROR);
                Poll::Pending
            }
            Err(err) if err.is_instance_of::<PyStopIteration>(py) => {
                Poll::Ready(err.value(py).getattr(intern!(py, "value")).map(Into::into))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl Wake for CoroutineWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        Python::with_gil(|gil| match FUTURE_OR_POLL.with(|cell| cell.take()) {
            Some(FutureOrPoll::Future(fut)) => FUTURE_OR_POLL
                .with(|cell| cell.set(Some(FutureOrPoll::Poll(self.delegate(fut.as_ref(gil)))))),
            Some(FutureOrPoll::Poll(_)) => unreachable!(),
            None => match self.state.get_or_init(gil, || State::Waken) {
                State::Waken => {}
                State::Delegated(_) => panic!("{}", MIXED_AWAITABLE_AND_FUTURE_ERROR),
                State::Pending(waker) => waker.wake(gil).expect("wake error"),
            },
        })
    }
}
