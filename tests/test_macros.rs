use pyo3::prelude::*;

macro_rules! make_struct_using_macro {
    // Ensure that one doesn't need to fall back on the escape type: tt
    // in order to macro create pyclass.
    ($className:path) => {
        #[pyclass(name=$className)]
        struct MyClass {}
    };
}

make_struct_using_macro!(MyClass);
