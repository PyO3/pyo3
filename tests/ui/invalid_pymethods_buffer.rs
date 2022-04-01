use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__getbuffer__")]
    fn getbuffer_must_be_unsafe(&self, _view: *mut pyo3::ffi::Py_buffer, _flags: std::os::raw::c_int) {}
}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__releasebuffer__")]
    fn releasebuffer_must_be_unsafe(&self, _view: *mut pyo3::ffi::Py_buffer) {}
}

fn main() {}
