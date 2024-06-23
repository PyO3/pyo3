#[cfg(all(feature = "instrumentation", not(Py_LIMITED_API), not(PyPy)))]
mod tests {
    use pyo3::instrumentation::{register_profiler, ProfileEvent, Profiler};
    use pyo3::prelude::*;
    use pyo3::pyclass;
    use pyo3::types::{PyFrame, PyList};

    #[pyclass]
    struct BasicProfiler {
        events: Py<PyList>,
    }

    impl Profiler for BasicProfiler {
        fn profile(&mut self, frame: Bound<'_, PyFrame>, event: ProfileEvent<'_>) -> PyResult<()> {
            let py = frame.py();
            let events = self.events.bind(py);
            match event {
                ProfileEvent::Call => events.append("call")?,
                ProfileEvent::Return(_) => events.append("return")?,
                _ => {}
            };
            Ok(())
        }
    }

    const PYTHON_CODE: &str = r#"
def foo():
    return "foo"

foo()
"#;

    #[test]
    fn test_profiler() {
        Python::with_gil(|py| {
            let events = PyList::empty_bound(py);
            let profiler = Bound::new(
                py,
                BasicProfiler {
                    events: events.clone().into(),
                },
            )
            .unwrap();
            register_profiler(profiler);

            py.run_bound(PYTHON_CODE, None, None).unwrap();

            assert_eq!(
                events.extract::<Vec<String>>().unwrap(),
                vec!["call", "call", "return", "return"]
            );
        })
    }
}
