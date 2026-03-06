use pyo3::prelude::*;

#[pyclass(frozen)]
pub struct Foo {
    #[pyo3(get)]
    field: u32,
}

#[pymethods]
//~^ ERROR: type mismatch resolving `<Foo as PyClass>::Frozen == False`
impl Foo {
    fn mut_method(&mut self) {}
//~^ ERROR: type mismatch resolving `<Foo as PyClass>::Frozen == False`
}

fn borrow_mut_fails(foo: Py<Foo>, py: Python) {
    let borrow = foo.bind(py).borrow_mut();
//~^ ERROR: type mismatch resolving `<Foo as PyClass>::Frozen == False`
}

#[pyclass(subclass)]
struct MutableBase;

#[pyclass(frozen, extends = MutableBase)]
struct ImmutableChild;

fn borrow_mut_of_child_fails(child: Py<ImmutableChild>, py: Python) {
    let borrow = child.bind(py).borrow_mut();
//~^ ERROR: type mismatch resolving `<ImmutableChild as PyClass>::Frozen == False`
}

fn py_get_of_mutable_class_fails(class: Py<MutableBase>) {
    class.get();
//~^ ERROR: type mismatch resolving `<MutableBase as PyClass>::Frozen == True`
}

fn pyclass_get_of_mutable_class_fails(class: &Bound<'_, MutableBase>) {
    class.get();
//~^ ERROR: type mismatch resolving `<MutableBase as PyClass>::Frozen == True`
}

#[pyclass(frozen)]
pub struct SetOnFrozenClass {
    #[pyo3(set)]
//~^ ERROR: cannot use `#[pyo3(set)]` on a `frozen` class
    field: u32,
}

fn main() {}
