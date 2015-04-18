#![crate_type = "dylib"]

#[macro_use] extern crate cpython;

use cpython::{PyObject, PyResult, PyModule, Python, PyTuple};

py_module_initializer!("hello", inithello, |py, m| {
    try!(m.add("__doc__", "Module documentation string"));
    try!(m.add("run", py_func!(py, run)));
    try!(add_val(py, &m));
    Ok(())
});

fn run<'p>(py: Python<'p>, args: &PyTuple<'p>) -> PyResult<'p, PyObject<'p>> {
    println!("Rust says: Hello Python!");
    for arg in args {
        println!("Rust got {}", arg);
    }
    Ok(py.None())
}

fn val<'p>(_: Python<'p>, _: &PyTuple<'p>) -> PyResult<'p, i32> {
    Ok(42)
}

// Workaround for Rust #24561
fn add_val<'p>(py: Python<'p>, m: &PyModule<'p>) -> PyResult<'p, ()> {
    m.add("val", py_func!(py, val))
}

