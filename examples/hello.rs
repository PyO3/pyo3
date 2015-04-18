extern crate cpython;

use cpython::{PythonObject, Python};
 
fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = py.import("sys").unwrap();
    let version: String = sys.get("version").unwrap().extract().unwrap();
    println!("Hello Python {}", version);
}

