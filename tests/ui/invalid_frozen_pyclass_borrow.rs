use pyo3::prelude::*;

#[pyclass(frozen)]
pub struct Foo {
    #[pyo3(get)]
    field: u32,
}

#[pymethods]
impl Foo {
    fn mut_method(&mut self) {}
}

fn borrow_mut_fails(foo: PyDetached<Foo>, py: Python) {
    let borrow = foo.as_ref(py).borrow_mut();
}

#[pyclass(subclass)]
struct MutableBase;

#[pyclass(frozen, extends = MutableBase)]
struct ImmutableChild;

fn borrow_mut_of_child_fails(child: PyDetached<ImmutableChild>, py: Python) {
    let borrow = child.as_ref(py).borrow_mut();
}

fn py_get_of_mutable_class_fails(class: PyDetached<MutableBase>) {
    class.get();
}

fn pyclass_get_of_mutable_class_fails(class: &PyCell<MutableBase>) {
    class.get();
}

#[pyclass(frozen)]
pub struct SetOnFrozenClass {
    #[pyo3(set)]
    field: u32,
}

fn main() {}
