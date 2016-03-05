#![crate_type = "dylib"]
#![feature(const_fn)]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate cpython;

use cpython::{Python, PyObject, PyRustObject, PyRustType, PyResult, GILProtected};
use std::cell::RefCell;

static MY_TYPE: GILProtected<RefCell<Option<PyRustType<i32>>>> = GILProtected::new(RefCell::new(None));

py_module_initializer!(custom_type, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    *MY_TYPE.get(py).borrow_mut() = Some(try!(m.add_type::<i32>(py, "MyType")
        .add("a", py_method!(a()))
        .set_new(py_fn!(new(arg: i32)))
        .finish()));
    Ok(())
});

fn new(py: Python, arg: i32) -> PyResult<PyRustObject<i32>> {
    Ok(MY_TYPE.get(py).borrow().as_ref().unwrap().create_instance(py, arg, ()))
}

fn a(py: Python, slf: &PyRustObject<i32>) -> PyResult<PyObject> {
    println!("a() was called with self={:?}", slf.get(py));
    Ok(py.None())
}

