#![crate_type = "dylib"]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{PythonObject, PyObject, PyRustObject, PyTuple, PyResult};

py_module_initializer!(custom_type, |_py, m| {
    try!(m.add("__doc__", "Module documentation string"));
    try!(m.add_type::<i32>("MyType")
        .add("a", py_method!(a))
        .finish());
    Ok(())
});

fn a<'p>(slf: &PyRustObject<'p, i32>, args: &PyTuple<'p>) -> PyResult<'p, PyObject<'p>> {
    println!("a() was called with self={:?} and args={:?}", slf.get(), args.as_object());
    Ok(slf.python().None())
}

