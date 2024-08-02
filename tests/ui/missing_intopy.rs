struct Blah;

#[pyo3::pyfunction]
fn blah() -> Blah {
    Blah
}

fn main() {}
