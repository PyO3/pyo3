use pyo3::buffer::{PyBufferRequest, PyBufferView};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn main() {
    Python::attach(|py| {
        let bytes = PyBytes::new(py, &[1, 2, 3]);
        PyBufferView::<u8>::with(&bytes, |view| {
            view.as_contiguous_slice(py);
            //~^ ERROR: C contiguity is not guaranteed by the requested buffer flags
            view.as_fortran_contiguous_slice(py);
            //~^ ERROR: Fortran contiguity is not guaranteed by the requested buffer flags
        })
        .unwrap();

        PyBufferView::<u8>::with_flags(&bytes, PyBufferRequest::simple().writable(), |view| {
            view.as_contiguous_mut_slice(py);
            //~^ ERROR: C contiguity is not guaranteed by the requested buffer flags
            view.as_fortran_contiguous_mut_slice(py);
            //~^ ERROR: Fortran contiguity is not guaranteed by the requested buffer flags
        })
        .unwrap();

        PyBufferView::<u8>::with_flags(&bytes, PyBufferRequest::simple().c_contiguous().format(), |view| {
            view.as_contiguous_mut_slice(py);
            //~^ ERROR: writability is not guaranteed by the requested buffer flags
        })
        .unwrap();

        PyBufferView::<u8>::with_flags(&bytes, PyBufferRequest::simple().f_contiguous().format(), |view| {
            view.as_fortran_contiguous_mut_slice(py);
            //~^ ERROR: writability is not guaranteed by the requested buffer flags
        })
        .unwrap();
    });
}
