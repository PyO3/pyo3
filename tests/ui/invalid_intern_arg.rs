use pyo3::Python;

fn main() {
    let _foo = if true { "foo" } else { "bar" };
    Python::attach(|py| py.import(pyo3::intern!(py, _foo)).unwrap());
//~^ ERROR: attempt to use a non-constant value in a constant
//~| ERROR: lifetime may not live long enough
}
