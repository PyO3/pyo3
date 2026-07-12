use pyo3::buffer::{PyBufferRequest, PyUntypedBufferView};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn main() {
    let _ = PyBufferRequest::simple().c_contiguous().f_contiguous();
    //~^ ERROR: the method `f_contiguous` exists
    let _ = PyBufferRequest::strided().format().format();
    //~^ ERROR: the method `format` exists
    let _ = PyBufferRequest::simple().indirect().indirect();
    //~^ ERROR: the method `indirect` exists

    Python::attach(|py| {
        let bytes = PyBytes::new(py, &[1, 2, 3]);
        PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::strided(), |view| {
            view.format();
            //~^ ERROR: format information is not available with the requested buffer flags
        })
        .unwrap();
    });
}
