use crate::inspect::fields::FieldInfo;

/// Information about a Python class.
#[derive(Debug)]
pub struct ClassInfo<'a> {
    /// Base information about the class.
    pub class: &'a ClassStructInfo<'a>,

    /// Information found in `#[pymethods]`.
    pub fields: &'a [&'a FieldInfo<'a>],
}

/// Subset of available information about a Python class, including only what is available by parsing the `#[pyclass]`
/// block (methods are missing).
#[derive(Debug)]
pub struct ClassStructInfo<'a> {
    pub name: &'a str,
    pub base: Option<&'a str>,
    pub fields: &'a [&'a FieldInfo<'a>],
}

impl<'a> ClassInfo<'a> {
    /// The Python name of this class.
    pub fn name(&'a self) -> &'a str {
        self.class.name
    }

    /// The Python's base class.
    pub fn base(&'a self) -> Option<&'a str> {
        self.class.base
    }

    /// All fields of the class.
    ///
    /// This includes:
    /// - struct attributes annotated with `#[getter]` or `#[setter]`
    /// - methods that appear in a `#[pymethods]` block
    pub fn fields(&'a self) -> impl Iterator<Item=&'a &'a FieldInfo<'a>> + 'a {
        self.class.fields
            .iter()
            .chain(self.fields)
    }
}

pub trait InspectClass<'a> {
    fn inspect() -> ClassInfo<'a>;
}

pub trait InspectStruct<'a> {
    fn inspect_struct() -> &'a ClassStructInfo<'a>;
}

pub trait InspectImpl<'a> {
    fn inspect_impl() -> &'a [&'a FieldInfo<'a>];
}

impl<'a, T> InspectClass<'a> for T
    where T: InspectStruct<'a>, T: InspectImpl<'a>
{
    fn inspect() -> ClassInfo<'a> {
        ClassInfo {
            class: Self::inspect_struct(),
            fields: Self::inspect_impl(),
        }
    }
}
