#![crate_type = "dylib"]
#![feature(const_fn)]

#[macro_use] extern crate cpython;

use cpython::{Python, PyObject, PyResult, GILProtected};
use cpython::rustobject::{PyRustType, PyRustObject};
use std::cell::RefCell;

static MY_TYPE: GILProtected<RefCell<Option<PyRustType<i32>>>> = GILProtected::new(RefCell::new(None));

py_module_initializer!(custom_type, initcustom_type, PyInit_custom_type, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    let mut b = m.add_type::<i32>(py, "MyType");
    b.add("a", py_method!(a()));
    b.set_new(py_fn!(new(arg: i32)));
    *MY_TYPE.get(py).borrow_mut() = Some(try!(b.finish()));
    Ok(())
});

fn new(py: Python, arg: i32) -> PyResult<PyRustObject<i32>> {
    Ok(MY_TYPE.get(py).borrow().as_ref().unwrap().create_instance(py, arg, ()))
}

fn a(py: Python, slf: &PyRustObject<i32>) -> PyResult<PyObject> {
    println!("a() was called with self={:?}", slf.get(py));
    Ok(py.None())
}

