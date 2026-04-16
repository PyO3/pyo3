use pyo3::buffer::PyBufferRequest;

fn main() {
    let _ = PyBufferRequest::simple().indirect().indirect();
}
