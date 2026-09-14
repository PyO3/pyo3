//@revisions: default nostd inspect
//@[default,nostd] no-experimental-inspect
//@[inspect] with-experimental-inspect
//@[default,inspect] with-std
//@[nostd] no-std

struct Blah;

#[pyo3::pyfunction]
fn blah() -> Blah {
    //~^ ERROR: `Blah` cannot be converted to a Python object
    //~[inspect]| ERROR: the trait bound `Blah: pyo3::impl_::introspection::return_type::Sealed` is not satisfied
    Blah
}

fn main() {}
