#[cfg(all(feature = "instrumentation", not(Py_LIMITED_API), not(PyPy)))]
mod tests {
    use pyo3::instrumentation::{setprofile, ProfileEvent, Profiler};
    use pyo3::prelude::*;
    use pyo3::pyclass;
    use pyo3::types::{PyFrame, PyList};

    #[pyclass]
    struct BasicProfiler {
        events: Py<PyList>,
    }

    impl Profiler for BasicProfiler {
        fn profile(&self, frame: Bound<'_, PyFrame>, event: ProfileEvent<'_>) -> PyResult<()> {
            let py = frame.py();
            let events = self.events.bind(py);
            match event {
                ProfileEvent::Call => events.append("call")?,
                ProfileEvent::Return(_) => events.append("return")?,
                ProfileEvent::CCall(_) => events.append("c call")?,
                ProfileEvent::CReturn(_) => events.append("c return")?,
                ProfileEvent::CException(_) => events.append("c exception")?,
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
    fn test_profiler_python() {
        Python::with_gil(|py| {
            let events = PyList::empty_bound(py);
            let profiler = Bound::new(
                py,
                BasicProfiler {
                    events: events.clone().into(),
                },
            )
            .unwrap();
            setprofile(profiler);

            py.run_bound(PYTHON_CODE, None, None).unwrap();

            assert_eq!(
                events.extract::<Vec<String>>().unwrap(),
                vec!["call", "call", "return", "return"]
            );
        })
    }

    const C_CALL_CODE: &str = r#"
import json

json.dumps([1, 2])
json.dumps()
"#;

    #[test]
    fn test_profiler_c() {
        Python::with_gil(|py| {
            let events = PyList::empty_bound(py);
            let profiler = Bound::new(
                py,
                BasicProfiler {
                    events: events.clone().into(),
                },
            )
            .unwrap();
            setprofile(profiler);

            let _ = py.run_bound(C_CALL_CODE, None, None);

            let events = events.extract::<Vec<String>>().unwrap();
            assert!(events.contains(&"c call".to_string()));
            assert!(events.contains(&"c return".to_string()));
            assert!(events.contains(&"c exception".to_string()));
        })
    }
}
