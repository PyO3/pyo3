//@revisions: default inspect
//@[default] without-experimental-inspect
//@[inspect] with-experimental-inspect

struct Blah;

#[pyo3::pyfunction]
fn blah() -> Blah {
    //~^ ERROR: `Blah` cannot be converted to a Python object
    //~| ERROR: no method named `map_err` found for struct `Blah` in the current scope
    //~[inspect]| ERROR: the trait bound `Blah: pyo3::impl_::introspection::return_type::Sealed` is not satisfied
    Blah
}

fn main() {}
