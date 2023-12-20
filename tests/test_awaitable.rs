#![cfg(feature = "macros")]

use std::task::Poll;

use futures::{future::poll_fn, FutureExt};
use pyo3::{
    coroutine::CancelHandle,
    exceptions::{PyAttributeError, PyTypeError},
    prelude::*,
    py_run,
    types::PyFuture,
};

#[path = "../src/tests/common.rs"]
mod common;

#[pyfunction]
async fn wrap_awaitable(awaitable: PyObject) -> PyResult<PyObject> {
    let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
    future.await
}

#[test]
fn awaitable() {
    Python::with_gil(|gil| {
        let wrap_awaitable = wrap_pyfunction!(wrap_awaitable, gil).unwrap();
        let test = r#"
        import types
        import asyncio;
        
        class BadAwaitable:
            def __await__(self):
                raise AttributeError("__await__")
        
        @types.coroutine
        def gen_coro():
            yield None
            
        async def main():
            await wrap_awaitable(...)
        asyncio.run(main())
        "#;
        let globals = gil.import("__main__").unwrap().dict();
        globals.set_item("wrap_awaitable", wrap_awaitable).unwrap();
        let run = |awaitable| {
            gil.run(
                &common::asyncio_windows(test).replace("...", awaitable),
                Some(globals),
                None,
            )
        };
        run("asyncio.sleep(0.001)").unwrap();
        run("gen_coro()").unwrap();
        assert!(run("None").unwrap_err().is_instance_of::<PyTypeError>(gil));
        assert!(run("BadAwaitable()")
            .unwrap_err()
            .is_instance_of::<PyAttributeError>(gil));
    })
}

#[test]
fn cancel_delegation() {
    #[pyfunction]
    async fn wrap_cancellable(awaitable: PyObject, #[pyo3(cancel_handle)] cancel: CancelHandle) {
        let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable)).unwrap();
        let result = future.await;
        Python::with_gil(|gil| {
            assert_eq!(
                result.unwrap_err().get_type(gil).name().unwrap(),
                "CancelledError"
            )
        });
        assert!(!cancel.is_cancelled());
    }
    Python::with_gil(|gil| {
        let wrap_cancellable = wrap_pyfunction!(wrap_cancellable, gil).unwrap();
        let test = r#"
        import asyncio;

        async def main():
            task = asyncio.create_task(wrap_cancellable(asyncio.sleep(0.001)))
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = gil.import("__main__").unwrap().dict();
        globals
            .set_item("wrap_cancellable", wrap_cancellable)
            .unwrap();
        gil.run(&common::asyncio_windows(test), Some(globals), None)
            .unwrap();
    })
}

#[test]
#[should_panic(expected = "PyFuture must be awaited in coroutine context")]
fn pyfuture_without_coroutine() {
    #[pyfunction]
    fn block_on(awaitable: PyObject) -> PyResult<PyObject> {
        let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
        futures::executor::block_on(future)
    }
    Python::with_gil(|gil| {
        let block_on = wrap_pyfunction!(block_on, gil).unwrap();
        let test = r#"
        async def coro():
            ...
        block_on(coro())
        "#;
        py_run!(gil, block_on, &common::asyncio_windows(test));
    })
}

async fn checkpoint() {
    let mut ready = false;
    poll_fn(|cx| {
        if ready {
            return Poll::Ready(());
        }
        ready = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await
}

#[test]
#[should_panic(expected = "Python awaitable mixed with Rust future")]
fn pyfuture_in_select() {
    #[pyfunction]
    async fn select(awaitable: PyObject) -> PyResult<PyObject> {
        let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
        futures::select_biased! {
            _ = checkpoint().fuse() => unreachable!(),
            res = future.fuse() => res,
        }
    }
    Python::with_gil(|gil| {
        let select = wrap_pyfunction!(select, gil).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            return await select(asyncio.sleep(1))
        asyncio.run(main())
        "#;
        let globals = gil.import("__main__").unwrap().dict();
        globals.set_item("select", select).unwrap();
        gil.run(&common::asyncio_windows(test), Some(globals), None)
            .unwrap();
    })
}

#[test]
#[should_panic(expected = "Python awaitable mixed with Rust future")]
fn pyfuture_in_select2() {
    #[pyfunction]
    async fn select2(awaitable: PyObject) -> PyResult<PyObject> {
        let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
        futures::select_biased! {
            res = future.fuse() => res,
            _ = checkpoint().fuse() => unreachable!(),
        }
    }
    Python::with_gil(|gil| {
        let select2 = wrap_pyfunction!(select2, gil).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            return await select2(asyncio.sleep(1))
        asyncio.run(main())
        "#;
        let globals = gil.import("__main__").unwrap().dict();
        globals.set_item("select2", select2).unwrap();
        gil.run(&common::asyncio_windows(test), Some(globals), None)
            .unwrap();
    })
}
