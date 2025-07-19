#[crate::pyclass]
#[pyo3(crate = "crate")]
#[derive(::std::clone::Clone)]
pub struct Foo;

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo2;

#[cfg_attr(any(Py_3_9, not(Py_LIMITED_API)), crate::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    unsendable,
    subclass,
    extends = crate::types::PyAny,
    module = "Spam",
    weakref,
    dict
))]
#[cfg_attr(not(any(Py_3_9, not(Py_LIMITED_API))), crate::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    unsendable,
    subclass,
    extends = crate::types::PyAny,
    module = "Spam"
))]
#[pyo3(crate = "crate")]
pub struct Bar {
    #[pyo3(get, set)]
    a: u8,
    #[pyo3(get, set)]
    b: Foo,
    #[pyo3(set)]
    c: ::std::option::Option<crate::Py<Foo2>>,
}

#[crate::pyclass(eq, eq_int)]
#[pyo3(crate = "crate")]
#[derive(PartialEq)]
pub enum Enum {
    Var0,
}

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo3 {
    #[pyo3(get, set)]
    #[cfg(any())]
    field: i32,

    #[pyo3(get, set)]
    #[cfg(not(any()))]
    field: u32,
}

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo4 {
    #[pyo3(get, set)]
    #[cfg(any())]
    #[cfg(not(any()))]
    field: i32,

    #[pyo3(get, set)]
    #[cfg(not(any()))]
    field: u32,
}

#[crate::pyclass(eq, ord)]
#[pyo3(crate = "crate")]
#[derive(PartialEq, PartialOrd)]
pub struct PointEqOrd {
    x: u32,
    y: u32,
    z: u32,
}

#[crate::pyclass(eq, ord)]
#[pyo3(crate = "crate")]
#[derive(PartialEq, PartialOrd)]
pub enum ComplexEnumEqOrd {
    Variant1 { a: u32, b: u32 },
    Variant2 { c: u32 },
}

#[crate::pyclass(eq, ord)]
#[pyo3(crate = "crate")]
#[derive(PartialEq, PartialOrd)]
pub enum TupleEnumEqOrd {
    Variant1(u32, u32),
    Variant2(u32),
}

#[crate::pyclass(crate = "crate")]
pub enum ComplexEnumManyVariantFields {
    ManyStructFields {
        field_1: u16,
        field_2: u32,
        field_3: u32,
        field_4: i32,
        field_5: u32,
        field_6: u32,
        field_7: u8,
        field_8: u32,
        field_9: i32,
        field_10: u32,
        field_11: u32,
        field_12: u32,
        field_13: u32,
        field_14: i16,
        field_15: u32,
    },
    ManyTupleFields(
        u16,
        u32,
        u32,
        i32,
        u32,
        u32,
        u8,
        u32,
        i32,
        u32,
        u32,
        u32,
        u32,
        i16,
        u32,
    ),
}

#[crate::pyclass(str = "{x}, {y}, {z}")]
#[pyo3(crate = "crate")]
pub struct PointFmt {
    x: u32,
    y: u32,
    z: u32,
}

#[crate::pyclass(str)]
#[pyo3(crate = "crate")]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl ::std::fmt::Display for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
