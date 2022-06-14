#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::inspect::classes::InspectClass;

mod common;

#[pyclass]
#[derive(Clone)]
struct Simple {}

#[pymethods]
impl Simple {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn plus_one(&self, a: usize) -> usize {
        a + 1
    }
}

#[test]
fn compiles() {
    // Nothing to do: if we reach this point, the compilation was successful :)
}

#[test]
fn simple_info() {
    let class_info = Simple::inspect();
    println!("Type of usize: {:?}", usize::type_input());
    println!("Type of class: {:?}", Simple::type_output());
    println!("Class:  {:?}", class_info);

    assert!(false)
}

#[test]
fn types() {
    assert_eq!("bool", format!("{}", <bool>::type_output()));
    assert_eq!("bytes", format!("{}", <&[u8]>::type_output()));
    assert_eq!("str", format!("{}", <String>::type_output()));
    assert_eq!("str", format!("{}", <char>::type_output()));
    assert_eq!("Optional[str]", format!("{}", <Option<String>>::type_output()));
    assert_eq!("Simple", format!("{}", <&PyCell<Simple>>::type_input()));
}
