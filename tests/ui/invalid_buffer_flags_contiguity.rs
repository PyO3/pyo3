use pyo3::buffer::PyBufferRequest;

fn main() {
    let _ = PyBufferRequest::simple().c_contiguous().f_contiguous();
}
