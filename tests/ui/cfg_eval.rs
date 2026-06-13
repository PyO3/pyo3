//@check-pass

#[pyo3::pyclass]
pub struct MyTupleStruct(
    #[cfg_attr(true, pyo3(get, name = "raw"))]
    u8,
);

#[pyo3::pyclass]
pub enum MyEnum {
    One{
        a: i32,
        #[cfg(false)]
        b: usize,
    },
    Two {
        #[cfg_attr(any(), pyo3(get))]
        field: u8,
    },
    #[cfg(all())]
    Three{
        y: String,
    },
}

#[pyo3::pyclass]
pub struct MyFieldStruct{
    #[cfg_attr(true, pyo3(get, name = "raw"))]
    pub f: u8,
    #[cfg_attr(any(), pyo3(set, name = "what"))]
    pub g: i32,
}
