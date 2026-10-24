//@with-experimental-inspect

pyo3::pyprotocol! {
    WithBody {
        fn method(&self) {}
        //~^ ERROR: protocol methods cannot have a body
    }
}

pyo3::pyprotocol! {
    WithoutSelf {
        fn method(x: i32);
        //~^ ERROR: protocol methods must take `self`
    }
}

pyo3::pyprotocol! {
    UnsupportedOption {
        #[pyo3(pass_module)]
        //~^ ERROR: only `name` and `signature` are supported on protocol methods
        fn method(&self);
    }
}

fn main() {}
