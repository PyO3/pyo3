#![cfg(feature = "experimental-async")]
#![cfg(not(target_arch = "wasm32"))]
use std::{ffi::CString, task::Poll, thread, time::Duration};

use futures::{channel::oneshot, future::poll_fn, FutureExt};
#[cfg(not(target_has_atomic = "64"))]
use portable_atomic::{AtomicBool, Ordering};
use pyo3::{
    coroutine::CancelHandle,
    prelude::*,
    py_run,
    types::{IntoPyDict, PyDict, PyType},
};
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::{AtomicBool, Ordering};

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
fn test_coroutine_qualname() {
    #[pyfunction]
    async fn my_fn() {}
    #[pyclass]
    struct MyClass;
    #[pymethods]
    impl MyClass {
        #[new]
        fn new() -> Self {
            Self
        }
        // TODO use &self when possible
        async fn my_method(_self: Py<Self>) {}
        #[classmethod]
        async fn my_classmethod(_cls: Py<PyType>) {}
        #[staticmethod]
        async fn my_staticmethod() {}
    }
    Python::with_gil(|gil| {
        let test = r#"
        for coro, name, qualname in [
            (my_fn(), "my_fn", "my_fn"),
            (MyClass().my_method(), "my_method", "MyClass.my_method"),
            #(MyClass().my_classmethod(), "my_classmethod", "MyClass.my_classmethod"),
            (MyClass.my_staticmethod(), "my_staticmethod", "MyClass.my_staticmethod"),
        ]:
            assert coro.__name__ == name and coro.__qualname__ == qualname
        "#;
        let locals = [
            ("my_fn", wrap_pyfunction!(my_fn, gil).unwrap().as_any()),
            ("MyClass", gil.get_type::<MyClass>().as_any()),
        ]
        .into_py_dict(gil)
        .unwrap();
        py_run!(gil, *locals, &handle_windows(test));
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

#[pyfunction]
async fn return_tuple() -> (usize, usize) {
    (42, 43)
}

#[test]
fn tuple_coroutine() {
    Python::with_gil(|gil| {
        let func = wrap_pyfunction!(return_tuple, gil).unwrap();
        let test = r#"import asyncio; assert asyncio.run(func()) == (42, 43)"#;
        py_run!(gil, func, &handle_windows(test));
    })
}

#[test]
fn cancelled_coroutine() {
    Python::with_gil(|gil| {
        let sleep = wrap_pyfunction!(sleep, gil).unwrap();
        let test = r#"
        import asyncio
        async def main():
            task = asyncio.create_task(sleep(999))
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = PyDict::new(gil);
        globals.set_item("sleep", sleep).unwrap();
        let err = gil
            .run(
                &CString::new(pyo3::unindent::unindent(&handle_windows(test))).unwrap(),
                Some(&globals),
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.value(gil).get_type().qualname().unwrap(),
            "CancelledError"
        );
    })
}

#[test]
fn coroutine_cancel_handle() {
    #[pyfunction]
    async fn cancellable_sleep(
        seconds: f64,
        #[pyo3(cancel_handle)] mut cancel: CancelHandle,
    ) -> usize {
        futures::select! {
            _ = sleep(seconds).fuse() => 42,
            _ = cancel.cancelled().fuse() => 0,
        }
    }
    Python::with_gil(|gil| {
        let cancellable_sleep = wrap_pyfunction!(cancellable_sleep, gil).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            task = asyncio.create_task(cancellable_sleep(999))
            await asyncio.sleep(0)
            task.cancel()
            return await task
        assert asyncio.run(main()) == 0
        "#;
        let globals = PyDict::new(gil);
        globals
            .set_item("cancellable_sleep", cancellable_sleep)
            .unwrap();
        gil.run(
            &CString::new(pyo3::unindent::unindent(&handle_windows(test))).unwrap(),
            Some(&globals),
            None,
        )
        .unwrap();
    })
}

#[test]
fn coroutine_is_cancelled() {
    #[pyfunction]
    async fn sleep_loop(#[pyo3(cancel_handle)] cancel: CancelHandle) {
        while !cancel.is_cancelled() {
            sleep(0.001).await;
        }
    }
    Python::with_gil(|gil| {
        let sleep_loop = wrap_pyfunction!(sleep_loop, gil).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            task = asyncio.create_task(sleep_loop())
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = PyDict::new(gil);
        globals.set_item("sleep_loop", sleep_loop).unwrap();
        gil.run(
            &CString::new(pyo3::unindent::unindent(&handle_windows(test))).unwrap(),
            Some(&globals),
            None,
        )
        .unwrap();
    })
}

#[test]
fn coroutine_panic() {
    #[pyfunction]
    async fn panic() {
        panic!("test panic");
    }
    Python::with_gil(|gil| {
        let panic = wrap_pyfunction!(panic, gil).unwrap();
        let test = r#"
        import asyncio
        coro = panic()
        try:
            asyncio.run(coro)
        except BaseException as err:
            assert type(err).__name__ == "PanicException"
            assert str(err) == "test panic"
        else:
            assert False
        try:
            coro.send(None)
        except RuntimeError as err:
            assert str(err) == "cannot reuse already awaited coroutine"
        else:
            assert False;
        "#;
        py_run!(gil, panic, &handle_windows(test));
    })
}

#[test]
fn test_async_method_receiver() {
    #[pyclass]
    struct Counter(usize);
    #[pymethods]
    impl Counter {
        #[new]
        fn new() -> Self {
            Self(0)
        }
        async fn get(&self) -> usize {
            self.0
        }
        async fn incr(&mut self) -> usize {
            self.0 += 1;
            self.0
        }
    }

    static IS_DROPPED: AtomicBool = AtomicBool::new(false);

    impl Drop for Counter {
        fn drop(&mut self) {
            IS_DROPPED.store(true, Ordering::SeqCst);
        }
    }

    Python::with_gil(|gil| {
        let test = r#"
        import asyncio

        obj = Counter()
        coro1 = obj.get()
        coro2 = obj.get()
        try:
            obj.incr()  # borrow checking should fail
        except RuntimeError as err:
            pass
        else:
            assert False
        assert asyncio.run(coro1) == 0
        coro2.close()
        coro3 = obj.incr()
        try:
            obj.incr()  # borrow checking should fail
        except RuntimeError as err:
            pass
        else:
            assert False
        try:
            obj.get() # borrow checking should fail
        except RuntimeError as err:
            pass
        else:
            assert False
        assert asyncio.run(coro3) == 1
        "#;
        let locals = [("Counter", gil.get_type::<Counter>())]
            .into_py_dict(gil)
            .unwrap();
        py_run!(gil, *locals, test);
    });

    assert!(IS_DROPPED.load(Ordering::SeqCst));
}

#[test]
fn test_async_method_receiver_with_other_args() {
    #[pyclass]
    struct Value(i32);
    #[pymethods]
    impl Value {
        #[new]
        fn new() -> Self {
            Self(0)
        }
        async fn get_value_plus_with(&self, v1: i32, v2: i32) -> i32 {
            self.0 + v1 + v2
        }
        async fn set_value(&mut self, new_value: i32) -> i32 {
            self.0 = new_value;
            self.0
        }
    }

    Python::with_gil(|gil| {
        let test = r#"
        import asyncio

        v = Value()
        assert asyncio.run(v.get_value_plus_with(3, 0)) == 3
        assert asyncio.run(v.set_value(10)) == 10
        assert asyncio.run(v.get_value_plus_with(1, 1)) == 12
        "#;
        let locals = [("Value", gil.get_type::<Value>())]
            .into_py_dict(gil)
            .unwrap();
        py_run!(gil, *locals, test);
    });
}
