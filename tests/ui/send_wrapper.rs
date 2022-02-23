use pyo3::prelude::*;
use pyo3::types::PyString;
use send_wrapper::SendWrapper;

fn main() {
    Python::with_gil(|py| {
        let string = PyString::new(py, "foo");

        let wrapped = SendWrapper::new(string);

        py.allow_threads(|| {
            let smuggled: &PyString = *wrapped;
            println!("{:?}", smuggled);
        });
    });
}