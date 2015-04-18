#[macro_use] extern crate cpython;

use cpython::{PythonObject, ObjectProtocol, PyModule, Python};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = PyModule::import(py, cstr!("sys")).unwrap();
    let path: String = sys.as_object().getattr("version").unwrap().extract().unwrap();
    println!("Hello Python {}", path);
}

