use pyo3::{
    pyclass, pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction_bound, Bound, PyResult,
};

#[pymodule]
pub fn enums(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SimpleEnum>()?;
    m.add_class::<ComplexEnum>()?;
    m.add_class::<TupleEnum>()?;
    m.add_class::<MixedComplexEnum>()?;
    m.add_wrapped(wrap_pyfunction_bound!(do_simple_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_complex_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_tuple_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_mixed_complex_stuff))?;
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
    Int {
        i: i32,
    },
    Float {
        f: f64,
    },
    Str {
        s: String,
    },
    EmptyStruct {},
    MultiFieldStruct {
        a: i32,
        b: f64,
        c: bool,
    },
    #[pyo3(constructor = (a = 42, b = None))]
    VariantWithDefault {
        a: i32,
        b: Option<String>,
    },
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
        ComplexEnum::VariantWithDefault { a, b } => ComplexEnum::VariantWithDefault {
            a: 2 * a,
            b: b.as_ref().map(|s| s.to_uppercase()),
        },
    }
}

#[pyclass]
pub enum TupleEnum {
    Full(i32, f64, bool),
    EmptyTuple(),
}

#[pyfunction]
pub fn do_tuple_stuff(thing: &TupleEnum) -> TupleEnum {
    match thing {
        TupleEnum::Full(a, b, c) => TupleEnum::Full(*a, *b, *c),
        TupleEnum::EmptyTuple() => TupleEnum::EmptyTuple(),
    }
}

#[pyclass]
pub enum MixedComplexEnum {
    Nothing {},
    Empty(),
}

#[pyfunction]
pub fn do_mixed_complex_stuff(thing: &MixedComplexEnum) -> MixedComplexEnum {
    match thing {
        MixedComplexEnum::Nothing {} => MixedComplexEnum::Empty (),
        MixedComplexEnum::Empty() => MixedComplexEnum::Nothing{},
    }
}
