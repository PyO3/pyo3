#![crate_type = "dylib"]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{PythonObject, PyObject, PyRustObject, PyResult};

py_module_initializer!(custom_type, |_py, m| {
    try!(m.add("__doc__", "Module documentation string"));
    try!(m.add_type::<i32>("MyType")
        .add("a", py_method!(a()))
        .finish());
    Ok(())
});

fn a<'p>(slf: &PyRustObject<'p, i32>) -> PyResult<'p, PyObject<'p>> {
    println!("a() was called with self={:?}", slf.get());
    Ok(slf.python().None())
}

