#![crate_type = "dylib"]
#![feature(const_fn)]

#![feature(trace_macros)]

#[macro_use] extern crate cpython;

use cpython::{Python, PyObject, PyRustObject, PyRustType, PyResult, GILProtected};
use std::cell::RefCell;

static MY_TYPE: GILProtected<RefCell<Option<PyRustType<i32>>>> = GILProtected::new(RefCell::new(None));

py_module_initializer!(custom_type, initcustom_type, PyInit_custom_type, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    try!(m.add_class::<MyType>(py));
    Ok(())
});

trace_macros!(true);

py_class!(class MyType, data: i32, |py| {

});

fn new(py: Python, arg: i32) -> PyResult<PyRustObject<i32>> {
    Ok(MY_TYPE.get(py).borrow().as_ref().unwrap().create_instance(py, arg, ()))
}

fn a(py: Python, slf: &PyRustObject<i32>) -> PyResult<PyObject> {
    println!("a() was called with self={:?}", slf.get(py));
    Ok(py.None())
}

