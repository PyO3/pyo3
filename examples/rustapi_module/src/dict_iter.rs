use pyo3::exceptions::RuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymodule]
fn test_dict(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<DictSize>()?;
    Ok(())
}

#[pyclass]
pub struct DictSize {
    expected: u32,
}

#[pymethods]
impl DictSize {
    #[new]
    fn new(obj: &PyRawObject, expected: u32) {
        obj.init(DictSize { expected })
    }

    fn iter_dict(&mut self, _py: Python<'_>, dict: &PyDict) -> PyResult<u32> {
        let mut seen = 0u32;
        for (sym, values) in dict.iter() {
            seen += 1;
            println!(
                "{:4}/{:4} iterations:{}=>{}",
                seen, self.expected, sym, values
            );
        }

        if seen == self.expected {
            Ok(seen)
        } else {
            Err(PyErr::new::<RuntimeError, _>(format!(
                "Expected {} iterations - performed {}",
                self.expected, seen
            )))
        }
    }
}
