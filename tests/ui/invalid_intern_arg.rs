use pyo3::Python;

fn main() {
    let _foo = if true { "foo" } else { "bar" };
    Python::with_gil(|py| py.import(pyo3::intern!(py, _foo)).unwrap());
}
