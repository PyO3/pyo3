use pyo3::prelude::*;

#[pyclass(immutable)]
pub struct Foo {
    #[pyo3(get)]
    field: u32,
}

fn borrow_mut_fails(foo: Py<Foo>, py: Python){
    let borrow = foo.as_ref(py).borrow_mut();
}

#[pyclass(subclass)]
struct MutableBase;

#[pyclass(immutable, extends = MutableBase)]
struct ImmutableChild;

fn borrow_mut_of_child_fails(child: Py<ImmutableChild>, py: Python){
    let borrow = child.as_ref(py).borrow_mut();
}

fn main(){}
