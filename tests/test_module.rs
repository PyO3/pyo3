#![feature(proc_macro, specialization, concat_idents)]

#[macro_use]
extern crate pyo3;

use pyo3::{PyDict, PyModule, PyObject, PyResult, Python};
use pyo3::py::{class, function, modinit};


#[class]
struct EmptyClass {}

fn sum_as_string(a: i64, b: i64) -> String {
    format!("{}", a + b).to_string()
}

#[function]
fn double(x: usize) -> usize {
    x * 2
}

/// This module is implemented in Rust.
#[modinit(module_with_functions)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a: i64, b: i64) -> PyResult<String> {
        let out = sum_as_string(a, b);
        return Ok(out);
    }

    #[pyfn(m, "no_parameters")]
    fn no_parameters() -> PyResult<usize> {
        return Ok(42);
    }

    m.add_class::<EmptyClass>();

    m.add("foo", "bar");

    add_function_to_module!(m, double, py);

    Ok(())
}

#[test]
fn test_module_with_functions() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("module_with_functions", unsafe { PyObject::from_owned_ptr(py, PyInit_module_with_functions()) }).unwrap();
    py.run("assert module_with_functions.__doc__.strip() == 'This module is implemented in Rust.'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.sum_as_string(1, 2) == '3'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.no_parameters() == 42", None, Some(d)).unwrap();
    py.run("assert module_with_functions.foo == 'bar'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.EmptyClass != None", None, Some(d)).unwrap();
    py.run("assert module_with_functions.double(3) == 6", None, Some(d)).unwrap();
}
