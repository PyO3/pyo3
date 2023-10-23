#![cfg(feature = "macros")]
#![cfg(not(target_arch = "wasm32"))]
use std::{task::Poll, thread, time::Duration};

use futures::{channel::oneshot, future::poll_fn};
use pyo3::{prelude::*, py_run};

#[path = "../src/tests/common.rs"]
mod common;

fn handle_windows(test: &str) -> String {
    let set_event_loop_policy = r#"
    import asyncio, sys
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    "#;
    pyo3::unindent::unindent(set_event_loop_policy) + &pyo3::unindent::unindent(test)
}

#[test]
fn noop_coroutine() {
    #[pyfunction]
    async fn noop() -> usize {
        42
    }
    Python::with_gil(|gil| {
        let noop = wrap_pyfunction!(noop, gil).unwrap();
        let test = "import asyncio; assert asyncio.run(noop()) == 42";
        py_run!(gil, noop, &handle_windows(test));
    })
}

#[test]
fn sleep_0_like_coroutine() {
    #[pyfunction]
    async fn sleep_0() -> usize {
        let mut waken = false;
        poll_fn(|cx| {
            if !waken {
                cx.waker().wake_by_ref();
                waken = true;
                return Poll::Pending;
            }
            Poll::Ready(42)
        })
        .await
    }
    Python::with_gil(|gil| {
        let sleep_0 = wrap_pyfunction!(sleep_0, gil).unwrap();
        let test = "import asyncio; assert asyncio.run(sleep_0()) == 42";
        py_run!(gil, sleep_0, &handle_windows(test));
    })
}

#[pyfunction]
async fn sleep(seconds: f64) -> usize {
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs_f64(seconds));
        tx.send(42).unwrap();
    });
    rx.await.unwrap()
}

#[test]
fn sleep_coroutine() {
    Python::with_gil(|gil| {
        let sleep = wrap_pyfunction!(sleep, gil).unwrap();
        let test = r#"import asyncio; assert asyncio.run(sleep(0.1)) == 42"#;
        py_run!(gil, sleep, &handle_windows(test));
    })
}

#[test]
fn cancelled_coroutine() {
    Python::with_gil(|gil| {
        let sleep = wrap_pyfunction!(sleep, gil).unwrap();
        let test = r#"
        import asyncio
        async def main():
            task = asyncio.create_task(sleep(1))
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = gil.import("__main__").unwrap().dict();
        globals.set_item("sleep", sleep).unwrap();
        let err = gil
            .run(
                &pyo3::unindent::unindent(&handle_windows(test)),
                Some(globals),
                None,
            )
            .unwrap_err();
        assert_eq!(err.value(gil).get_type().name().unwrap(), "CancelledError");
    })
}
