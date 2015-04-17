#![crate_type = "dylib"] 

#[macro_use] extern crate cpython;

use cpython::{PyModule, PyResult, Python};

py_module_initializer!("testmodule", inittestmodule, |py, m| {
    println!("in initializer");
    try!(m.add(cstr!("__doc__"), "Module documentation string"));
    try!(m.add(cstr!("__author__"), "Daniel Grunwald"));
    try!(m.add(cstr!("__version__"), "0.0.1"));
    Ok(())
});

