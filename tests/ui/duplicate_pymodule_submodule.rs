#[pyo3::pymodule]
mod mymodule {
	#[pyo3::pymodule(submodule)]
	mod submod {}
}

fn main() {}
