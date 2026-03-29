#[pyo3::pymodule]
//~^ ERROR: cannot find value `_PYO3_DEF` in module `submod`
mod mymodule {
    #[pyo3::pymodule(submodule)]
    mod submod {}
    //~^ ERROR: `submodule` may only be specified once (it is implicitly always specified for nested modules)
}

fn main() {}
