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
    Async {
        async fn method(&self);
        //~^ ERROR: protocol methods cannot be `async`
    }
}

pyo3::pyprotocol! {
    Generic {
        fn method<T>(&self, x: T);
        //~^ ERROR: Python functions cannot have generic type parameters
    }
}

pyo3::pyprotocol! {
    WithPython {
        fn method(&self, py: pyo3::Python<'_>);
        //~^ ERROR: protocol methods cannot take a `Python` token
    }
}

pyo3::pyprotocol! {
    PassModule {
        #[pyo3(pass_module)]
        //~^ ERROR: `pass_module` cannot be used on protocol methods
        fn method(&self);
    }
}

pyo3::pyprotocol! {
    #[pyo3(module = "foo")]
    //~^ ERROR: expected `name` or `crate`
    UnknownOption {}
}

fn main() {}
