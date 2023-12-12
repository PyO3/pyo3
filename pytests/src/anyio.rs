use std::{task::Poll, thread, time::Duration};

use futures::{channel::oneshot, future::poll_fn};
use pyo3::prelude::*;

#[pyfunction]
async fn sleep(seconds: f64, result: Option<PyObject>) -> Option<PyObject> {
    if seconds <= 0.0 {
        let mut ready = false;
        poll_fn(|cx| {
            if ready {
                return Poll::Ready(());
            }
            ready = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        })
        .await;
    } else {
        let (tx, rx) = oneshot::channel();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs_f64(seconds));
            tx.send(()).unwrap();
        });
        rx.await.unwrap();
    }
    result
}

#[pymodule]
pub fn anyio(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sleep, m)?)?;
    Ok(())
}
