use pyo3::{
    pyclass, pyfunction, pymodule, types::PyModule, wrap_pyfunction_bound, Bound, PyResult,
};

#[pymodule]
pub fn enums(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SimpleEnum>()?;
    m.add_class::<ComplexEnum>()?;
    m.add_wrapped(wrap_pyfunction_bound!(do_simple_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_complex_stuff))?;
    Ok(())
}

#[pyclass]
pub enum SimpleEnum {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[pyfunction]
pub fn do_simple_stuff(thing: &SimpleEnum) -> SimpleEnum {
    match thing {
        SimpleEnum::Sunday => SimpleEnum::Monday,
        SimpleEnum::Monday => SimpleEnum::Tuesday,
        SimpleEnum::Tuesday => SimpleEnum::Wednesday,
        SimpleEnum::Wednesday => SimpleEnum::Thursday,
        SimpleEnum::Thursday => SimpleEnum::Friday,
        SimpleEnum::Friday => SimpleEnum::Saturday,
        SimpleEnum::Saturday => SimpleEnum::Sunday,
    }
}

#[pyclass]
pub enum ComplexEnum {
    Int { i: i32 },
    Float { f: f64 },
    Str { s: String },
    EmptyStruct {},
    MultiFieldStruct { a: i32, b: f64, c: bool },
}

#[pyfunction]
pub fn do_complex_stuff(thing: &ComplexEnum) -> ComplexEnum {
    match thing {
        ComplexEnum::Int { i } => ComplexEnum::Str { s: i.to_string() },
        ComplexEnum::Float { f } => ComplexEnum::Float { f: f * f },
        ComplexEnum::Str { s } => ComplexEnum::Int { i: s.len() as i32 },
        ComplexEnum::EmptyStruct {} => ComplexEnum::EmptyStruct {},
        ComplexEnum::MultiFieldStruct { a, b, c } => ComplexEnum::MultiFieldStruct {
            a: *a,
            b: *b,
            c: *c,
        },
    }
}
