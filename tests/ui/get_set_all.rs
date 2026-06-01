use pyo3::prelude::*;

#[pyclass(set_all)]
//~^ ERROR: `set_all` on an unit struct does nothing, because unit structs have no fields
struct Foo;

#[pyclass(set_all)]
struct Foo2{
    #[pyo3(set)]
//~^ ERROR: useless `set` - the struct is already annotated with `set_all`
    field: u8,
}

#[pyclass(get_all)]
//~^ ERROR: `get_all` on an unit struct does nothing, because unit structs have no fields
struct Foo3;

#[pyclass(get_all)]
struct Foo4{
    #[pyo3(get)]
//~^ ERROR: useless `get` - the struct is already annotated with `get_all`
    field: u8,
}

fn main() {}
