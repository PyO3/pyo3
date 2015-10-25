extern crate cpython;

use cpython::Python;
use cpython::ObjectProtocol; //for call method

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get("version", py).unwrap().extract(py).unwrap();

    let os = py.import("os").unwrap();
    let getenv = os.get("getenv", py).unwrap();
    let user: String = getenv.call(("USER",), None, py).unwrap().extract(py).unwrap();

    println!("Hello {}, I'm Python {}", user, version);
}
