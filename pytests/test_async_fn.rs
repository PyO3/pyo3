use std::time::Duration;

use pyo3::prelude::*;

const TEST_MOD: &'static str = r#"
import asyncio 

async def py_sleep(duration):
    await asyncio.sleep(duration)

async def sleep(sleeper):
    await sleeper()

async def sleep_for(sleeper, duration):
    await sleeper(duration)
"#;

#[pyfunction]
async fn sleep() -> PyResult<PyObject> {
    async_std::task::sleep(Duration::from_secs(1)).await;
    Ok(Python::with_gil(|py| py.None()))
}

#[pyfunction]
async fn sleep_for(duration: PyObject) -> PyResult<PyObject> {
    let duration: f64 = Python::with_gil(|py| duration.as_ref(py).extract())?;
    let microseconds = duration * 1.0e6;

    async_std::task::sleep(Duration::from_micros(microseconds as u64)).await;

    Ok(Python::with_gil(|py| py.None()))
}

#[pyclass]
struct Sleeper {
    duration: Duration,
}

#[pymethods]
impl Sleeper {
    // FIXME: &self screws up the 'static requirement for into_coroutine. Could be fixed by
    // supporting impl Future along with async (which would be nice anyway). I don't think any
    // async method that accesses member variables can be reasonably supported with the async fn
    // syntax because of the 'static lifetime requirement, so it would have to fall back to
    // impl Future in nearly all cases
    //
    // async fn sleep(&self) -> PyResult<PyObject> {
    //     let duration = self.duration.clone();

    //     async_std::task::sleep(duration).await;

    //     Ok(Python::with_gil(|py| py.None()))
    // }
}

#[pyo3_asyncio::async_std::test]
async fn test_sleep() -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let sleeper_mod = PyModule::new(py, "rust_sleeper")?;
        sleeper_mod.add_wrapped(pyo3::wrap_pyfunction!(sleep))?;

        let test_mod =
            PyModule::from_code(py, TEST_MOD, "test_rust_coroutine/test_mod.py", "test_mod")?;

        pyo3_asyncio::into_future(test_mod.call_method1("sleep", (sleeper_mod.getattr("sleep")?,))?)
    })?;

    fut.await?;

    Ok(())
}

#[pyo3_asyncio::async_std::test]
async fn test_sleep_for() -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let sleeper_mod = PyModule::new(py, "rust_sleeper")?;
        sleeper_mod.add_wrapped(pyo3::wrap_pyfunction!(sleep_for))?;

        let test_mod =
            PyModule::from_code(py, TEST_MOD, "test_rust_coroutine/test_mod.py", "test_mod")?;

        pyo3_asyncio::into_future(test_mod.call_method1(
            "sleep_for",
            (sleeper_mod.getattr("sleep_for")?, 2.into_py(py)),
        )?)
    })?;

    fut.await?;

    Ok(())
}

// #[pyo3_asyncio::async_std::test]
// async fn test_sleeper() -> PyResult<()> {
//     let fut = Python::with_gil(|py| {
//         let sleeper = PyCell::new(
//             py,
//             Sleeper {
//                 duration: Duration::from_secs(3),
//             },
//         )?;

//         let test_mod =
//             PyModule::from_code(py, TEST_MOD, "test_rust_coroutine/test_mod.py", "test_mod")?;

//         pyo3_asyncio::into_future(test_mod.call_method1("sleep_for", (sleeper.getattr("sleep")?,))?)
//     })?;

//     fut.await?;

//     Ok(())
// }

pyo3_asyncio::testing::test_main!(
    #[pyo3_asyncio::async_std::main],
    "Async #[pyfunction] Test Suite"
);
