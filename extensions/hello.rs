#![crate_type = "dylib"]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{PyObject, PyResult, Python, PyTuple, PyDict};

py_module_initializer!(hello, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    try!(m.add(py, "run", py_fn!(run)));
    try!(m.add(py, "val", py_fn!(val())));
    Ok(())
});

fn run(py: Python, args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    println!("Rust says: Hello Python!");
    for arg in args.as_slice() {
        println!("Rust got {}", arg);
    }
    if let Some(kwargs) = kwargs {
        for (key, val) in kwargs.items(py) {
            println!("{} = {}", key, val);
        }
    }
    Ok(py.None())
}

fn val(_: Python) -> PyResult<i32> {
    Ok(42)
}

