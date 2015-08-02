extern crate cpython;

use cpython::{PythonObject, Python};
use cpython::ObjectProtocol; //for call method

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get("version").unwrap().extract().unwrap();

    let os = py.import("os").unwrap();
    let getenv = os.get("getenv").unwrap();
    let user: String = getenv.call(("USER",), None).unwrap().extract().unwrap();

    println!("Hello {}, I'm Python {}", user, version);
}
