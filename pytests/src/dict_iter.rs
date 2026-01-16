use pyo3::prelude::*;

#[pymodule]
pub mod dict_iter {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[pyclass]
    pub struct DictSize {
        expected: u32,
    }

    #[pymethods]
    impl DictSize {
        #[new]
        fn new(expected: u32) -> Self {
            DictSize { expected }
        }

        fn iter_dict(&mut self, _py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<u32> {
            let mut seen = 0u32;
            for (sym, values) in dict {
                seen += 1;
                println!(
                    "{:4}/{:4} iterations:{}=>{}",
                    seen, self.expected, sym, values
                );
            }

            if seen == self.expected {
                Ok(seen)
            } else {
                Err(PyErr::new::<PyRuntimeError, _>(format!(
                    "Expected {} iterations - performed {}",
                    self.expected, seen
                )))
            }
        }
    }
}
