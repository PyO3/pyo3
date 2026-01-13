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

mod test_utils;

fn handle_windows(test: &str) -> String {
    let set_event_loop_policy = r#"
    import asyncio, sys
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    "#;
    pyo3::impl_::unindent::unindent(set_event_loop_policy) + &pyo3::impl_::unindent::unindent(test)
}

#[test]
fn noop_coroutine() {
    #[pyfunction]
    async fn noop() -> usize {
        42
    }
    Python::attach(|py| {
        let noop = wrap_pyfunction!(noop, py).unwrap();
        let test = "import asyncio; assert asyncio.run(noop()) == 42";
        py_run!(py, noop, &handle_windows(test));
    })
}

#[test]
fn test_async_function_returns_unit_none() {
    #[pyfunction]
    async fn returns_unit() {}

    Python::attach(|py| {
        let returns_unit = wrap_pyfunction!(returns_unit, py).unwrap();
        let test = "import asyncio; assert asyncio.run(returns_unit()) is None";
        pyo3::py_run!(py, returns_unit, &handle_windows(test));
    });
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
    Python::attach(|py| {
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
            ("my_fn", wrap_pyfunction!(my_fn, py).unwrap().as_any()),
            ("MyClass", py.get_type::<MyClass>().as_any()),
        ]
        .into_py_dict(py)
        .unwrap();
        py_run!(py, *locals, &handle_windows(test));
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
    Python::attach(|py| {
        let sleep_0 = wrap_pyfunction!(sleep_0, py).unwrap();
        let test = "import asyncio; assert asyncio.run(sleep_0()) == 42";
        py_run!(py, sleep_0, &handle_windows(test));
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
    Python::attach(|py| {
        let sleep = wrap_pyfunction!(sleep, py).unwrap();
        let test = r#"import asyncio; assert asyncio.run(sleep(0.1)) == 42"#;
        py_run!(py, sleep, &handle_windows(test));
    })
}

#[pyfunction]
async fn return_tuple() -> (usize, usize) {
    (42, 43)
}

#[test]
fn tuple_coroutine() {
    Python::attach(|py| {
        let func = wrap_pyfunction!(return_tuple, py).unwrap();
        let test = r#"import asyncio; assert asyncio.run(func()) == (42, 43)"#;
        py_run!(py, func, &handle_windows(test));
    })
}

#[test]
fn cancelled_coroutine() {
    Python::attach(|py| {
        let sleep = wrap_pyfunction!(sleep, py).unwrap();
        let test = r#"
        import asyncio
        async def main():
            task = asyncio.create_task(sleep(999))
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = PyDict::new(py);
        globals.set_item("sleep", sleep).unwrap();
        let err = py
            .run(
                &CString::new(pyo3::impl_::unindent::unindent(&handle_windows(test))).unwrap(),
                Some(&globals),
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.value(py).get_type().qualname().unwrap(),
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
    Python::attach(|py| {
        let cancellable_sleep = wrap_pyfunction!(cancellable_sleep, py).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            task = asyncio.create_task(cancellable_sleep(999))
            await asyncio.sleep(0)
            task.cancel()
            return await task
        assert asyncio.run(main()) == 0
        "#;
        let globals = PyDict::new(py);
        globals
            .set_item("cancellable_sleep", cancellable_sleep)
            .unwrap();
        py.run(
            &CString::new(pyo3::impl_::unindent::unindent(&handle_windows(test))).unwrap(),
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
    Python::attach(|py| {
        let sleep_loop = wrap_pyfunction!(sleep_loop, py).unwrap();
        let test = r#"
        import asyncio;
        async def main():
            task = asyncio.create_task(sleep_loop())
            await asyncio.sleep(0)
            task.cancel()
            await task
        asyncio.run(main())
        "#;
        let globals = PyDict::new(py);
        globals.set_item("sleep_loop", sleep_loop).unwrap();
        py.run(
            &CString::new(pyo3::impl_::unindent::unindent(&handle_windows(test))).unwrap(),
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
    Python::attach(|py| {
        let panic = wrap_pyfunction!(panic, py).unwrap();
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
        py_run!(py, panic, &handle_windows(test));
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
        async fn get(&self, resolve: bool) -> usize {
            if !resolve {
                // hang the future to test borrow checking
                std::future::pending().await
            }
            self.0
        }
        async fn incr(&mut self, resolve: bool) -> usize {
            if !resolve {
                // hang the future to test borrow checking
                std::future::pending().await
            }
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

    Python::attach(|py| {
        let test = r#"
        import asyncio

        obj = Counter()

        assert asyncio.run(obj.get(True)) == 0
        assert asyncio.run(obj.incr(True)) == 1

        for left in [obj.get, obj.incr]:
            for right in [obj.get, obj.incr]:
                # first future will not resolve to hold the borrow
                coro1 = left(False)
                coro2 = right(True)
                try:
                    asyncio.run(asyncio.gather(coro1, coro2))
                except RuntimeError as err:
                    ran = False
                else:
                    ran = True
                if left is obj.incr or right is obj.incr:
                    assert not ran, "mutable method calls should not run concurrently with other method calls"
        "#;
        let locals = [("Counter", py.get_type::<Counter>())]
            .into_py_dict(py)
            .unwrap();
        py_run!(py, *locals, test);
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

    Python::attach(|py| {
        let test = r#"
        import asyncio

        v = Value()
        assert asyncio.run(v.get_value_plus_with(3, 0)) == 3
        assert asyncio.run(v.set_value(10)) == 10
        assert asyncio.run(v.get_value_plus_with(1, 1)) == 12
        "#;
        let locals = [("Value", py.get_type::<Value>())]
            .into_py_dict(py)
            .unwrap();
        py_run!(py, *locals, test);
    });
}

#[test]
fn test_async_fn_borrowed_values() {
    #[pyclass]
    struct Data {
        value: String,
    }
    #[pymethods]
    impl Data {
        #[new]
        fn new(value: String) -> Self {
            Self { value }
        }
        async fn borrow_value(&self) -> &str {
            &self.value
        }
        async fn borrow_value_or_default<'a>(&'a self, default: &'a str) -> &'a str {
            if self.value.is_empty() {
                default
            } else {
                &self.value
            }
        }
    }
    Python::attach(|py| {
        let test = r#"
        import asyncio

        v = Data('hello')
        assert asyncio.run(v.borrow_value()) == 'hello'
        assert asyncio.run(v.borrow_value_or_default('')) == 'hello'

        v_empty = Data('')
        assert asyncio.run(v_empty.borrow_value_or_default('default')) == 'default'
        "#;
        let locals = [("Data", py.get_type::<Data>())].into_py_dict(py).unwrap();
        py_run!(py, *locals, test);
    });
}

#[test]
fn test_async_fn_class_values() {
    #[pyclass]
    struct Value(i32);

    #[pymethods]
    impl Value {
        #[new]
        fn new(x: i32) -> Self {
            Self(x)
        }

        #[getter]
        fn value(&self) -> i32 {
            self.0
        }
    }

    #[pyfunction]
    async fn add_two_values(obj: &Value, obj2: &Value) -> Value {
        Value(obj.0 + obj2.0)
    }

    Python::attach(|py| {
        let test = r#"
        import asyncio

        v1 = Value(1)
        v2 = Value(2)
        assert asyncio.run(add_two_values(v1, v2)).value == 3
        "#;
        let locals = [
            ("Value", py.get_type::<Value>().into_any()),
            (
                "add_two_values",
                wrap_pyfunction!(add_two_values, py).unwrap().into_any(),
            ),
        ]
        .into_py_dict(py)
        .unwrap();
        py_run!(py, *locals, test);
    });
}
