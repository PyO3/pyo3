use pyo3::prelude::*;
use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;

#[pyclass]
pub struct DictSize {
    expected: usize,
}

#[pymethods]
impl DictSize {
    #[new]
    fn __new__(obj: &PyRawObject, expected: i32) -> PyResult<()> {
        obj.init(|_t|  DictSize{ expected:expected as usize })
    }

    fn iter_dict(&mut self, _py: Python, dict: &PyDict) -> PyResult<()> {
        let mut seen=0usize;
        for (sym, values) in dict.iter() {
            seen += 1;            
            println!("{:4}/{:4} iterations:{}=>{}",seen,self.expected,sym,values);
        }
        
        match seen == self.expected {
            true=>Ok(()),
            _ => Err(PyErr::new::<exc::TypeError, _>(format!("Expected {} iterations - performed {}",self.expected,seen)))
        }
    }
}
