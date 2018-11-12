use pyo3::prelude::*;

use pyo3::exceptions as exc;
use pyo3::types::PyDict;

#[pymodinit(_test_dict)]
fn test_dict(_py: Python, m: &PyModule) -> PyResult<()> {
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
    fn __new__(obj: &PyRawObject, expected: u32) -> PyResult<()> {
        obj.init(|_t| DictSize { expected })
    }

    fn iter_dict(&mut self, _py: Python, dict: &PyDict) -> PyResult<u32> {
        let mut seen = 0u32;
        for (sym, values) in dict.iter() {
            seen += 1;
            println!(
                "{:4}/{:4} iterations:{}=>{}",
                seen, self.expected, sym, values
            );
        }

        match seen == self.expected {
            true => Ok(seen),
            _ => Err(PyErr::new::<exc::TypeError, _>(format!(
                "Expected {} iterations - performed {}",
                self.expected, seen
            ))),
        }
    }
}
