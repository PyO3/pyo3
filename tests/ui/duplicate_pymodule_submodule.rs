#[pyo3::pymodule]
mod mymodule {
    #[pyo3::pymodule(submodule)]
    mod submod {}
    //~^ ERROR: `submodule` may only be specified once (it is implicitly always specified for nested modules)
}

fn main() {}
