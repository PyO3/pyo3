struct Blah;

#[pyo3::pyfunction]
fn blah() -> Blah {
//~^ ERROR: `Blah` cannot be converted to a Python object
//~| ERROR: no method named `map_err` found for struct `Blah` in the current scope
    Blah
}

fn main() {}
