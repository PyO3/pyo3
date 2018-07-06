#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use std::isize;

use pyo3::py::class;
use pyo3::py::methods;

#[macro_use]
mod common;

#[class]
struct ClassWithProperties {
    num: i32,
    token: PyToken,
}

#[methods]
impl ClassWithProperties {
    fn get_num(&self) -> PyResult<i32> {
        Ok(self.num)
    }

    #[getter(DATA)]
    fn get_data(&self) -> PyResult<i32> {
        Ok(self.num)
    }
    #[setter(DATA)]
    fn set_data(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
    }
}

#[test]
fn class_with_properties() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = py
        .init(|t| ClassWithProperties { num: 10, token: t })
        .unwrap();

    py_run!(py, inst, "assert inst.get_num() == 10");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
    py_run!(py, inst, "inst.DATA = 20");
    py_run!(py, inst, "assert inst.get_num() == 20");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
}

#[class]
struct GetterSetter {
    token: PyToken,
    #[prop(get, set)]
    num: i32,
    #[prop(get, set)]
    text: String,
}

#[methods]
impl GetterSetter {
    fn get_num2(&self) -> PyResult<i32> {
        Ok(self.num)
    }
}

#[test]
fn getter_setter_autogen() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst =
        py.init(|t| GetterSetter {
            num: 10,
            token: t,
            text: "Hello".to_string(),
        }).unwrap();

    py_run!(py, inst, "assert inst.num == 10");
    py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
    py_run!(
        py,
        inst,
        "assert inst.text == 'Hello'; inst.text = 'There'; assert inst.text == 'There'"
    );
}
