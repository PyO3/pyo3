#![allow(unused_variables)]
#![feature(proc_macro, specialization, const_fn)]
extern crate pyo3;
use pyo3::prelude::*;
use pyo3::py::{class, methods, modinit};


#[class]
struct MyClass{
    title: String, // has Clone trait
    // #[prop(get, set)]
    num: i32 // has Copy trait
}

#[methods]
impl MyClass{

    #[new]
    fn __new__(obj: &PyRawObject, title: String) -> PyResult<()> {
        obj.init(|t| MyClass{title: title, num: 1987})
    }

}


#[modinit(_my_class)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MyClass>()?;

    Ok(())
}
