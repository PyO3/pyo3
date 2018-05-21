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

    m.add_class::<EmptyClass>().unwrap();

    m.add("foo", "bar").unwrap();

    m.add_function(wrap_function!(double)).unwrap();
    m.add("also_double", wrap_function!(double)(py)).unwrap();

    Ok(())
}

#[test]
#[cfg(Py_3)]
fn test_module_with_functions() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("module_with_functions", unsafe { PyObject::from_owned_ptr(py, PyInit_module_with_functions()) }).unwrap();
    py.run("assert module_with_functions.__doc__ == 'This module is implemented in Rust.'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.sum_as_string(1, 2) == '3'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.no_parameters() == 42", None, Some(d)).unwrap();
    py.run("assert module_with_functions.foo == 'bar'", None, Some(d)).unwrap();
    py.run("assert module_with_functions.EmptyClass != None", None, Some(d)).unwrap();
    py.run("assert module_with_functions.double(3) == 6", None, Some(d)).unwrap();
    py.run("assert module_with_functions.also_double(3) == 6", None, Some(d)).unwrap();
}

#[test]
#[cfg(Py_3)]
fn test_module_from_code() {

    let gil = Python::acquire_gil();
    let py = gil.python();

    let adder_mod = PyModule::from_code(py, 
        "def add(a,b):\n\treturn a+b", 
        "adder_mod.py", 
        "adder_mod")
        .expect("Module code should be loaded");

    let add_func = adder_mod
        .get("add")
        .expect("Add fucntion should be in the module")
        .to_object(py);

    let ret_value: i32 = add_func
        .call1(py, (1, 2))
        .expect("A value should be returned")
        .extract(py)
        .expect("The value should be able to be converted to an i32");

    assert_eq!(ret_value, 3);
}
