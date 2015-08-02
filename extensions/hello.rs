#![crate_type = "dylib"]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{PyObject, PyResult, Python, PyTuple, PyDict};

py_module_initializer!(hello, |_py, m| {
    try!(m.add("__doc__", "Module documentation string"));
    try!(m.add("run", py_fn!(run)));
    try!(m.add("val", py_fn!(val())));
    Ok(())
});

fn run<'p>(py: Python<'p>, args: &PyTuple<'p>, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>> {
    println!("Rust says: Hello Python!");
    for arg in args {
        println!("Rust got {}", arg);
    }
    if let Some(kwargs) = kwargs {
        for (key, val) in kwargs.items() {
            println!("{} = {}", key, val);
        }
    }
    Ok(py.None())
}

fn val<'p>(_: Python<'p>) -> PyResult<'p, i32> {
    Ok(42)
}

