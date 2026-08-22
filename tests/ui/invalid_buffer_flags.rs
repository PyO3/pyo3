use pyo3::buffer::{PyBufferRequest, PyUntypedBufferView};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn main() {
    let _ = PyBufferRequest::simple().c_contiguous().f_contiguous();
    //~^ ERROR: contiguity has already been constrained for this buffer request
    let _ = PyBufferRequest::strided().format().format();
    //~^ ERROR: format information has already been requested for this buffer request
    let _ = PyBufferRequest::simple().indirect().indirect();
    //~^ ERROR: suboffsets can only be requested on a direct unconstrained buffer request

    Python::attach(|py| {
        let bytes = PyBytes::new(py, &[1, 2, 3]);
        PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::strided(), |view| {
            view.format();
            //~^ ERROR: format information is not available with the requested buffer flags
        })
        .unwrap();
    });
}
