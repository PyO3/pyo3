#![crate_type = "dylib"]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{PyObject, PyResult,Python, PyTuple};

py_module_initializer!(hello, |_py, m| {
    try!(m.add("__doc__", "Module documentation string"));
    try!(m.add("run", py_fn!(run)));
    try!(m.add("val", py_fn!(val)));
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

