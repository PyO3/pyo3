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
        //~^ ERROR: protocol methods must take `&self`
    }
}

pyo3::pyprotocol! {
    MutSelf {
        fn method(&mut self);
        //~^ ERROR: protocol methods must take `&self`
    }
}

pyo3::pyprotocol! {
    UnsupportedOption {
        #[pyo3(pass_module)]
        //~^ ERROR: expected `name` or `signature`
        fn method(&self);
    }
}

pyo3::pyprotocol! {
    #[pyo3(signature = ())]
    //~^ ERROR: expected `name` or `crate`
    UnsupportedProtocolOption {}
}

pyo3::pyprotocol! {
    DuplicateOption {
        #[pyo3(name = "a", name = "b")]
        //~^ ERROR: `name` may only be specified once
        fn method(&self);
    }
}

fn main() {}
