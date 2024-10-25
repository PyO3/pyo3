use pyo3::IntoPyObject;

#[derive(IntoPyObject)]
struct Foo();

#[derive(IntoPyObject)]
struct Foo2 {}

#[derive(IntoPyObject)]
enum EmptyEnum {}

#[derive(IntoPyObject)]
enum EnumWithEmptyTupleVar {
    EmptyTuple(),
    Valid(String),
}

#[derive(IntoPyObject)]
enum EnumWithEmptyStructVar {
    EmptyStruct {},
    Valid(String),
}

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct EmptyTransparentTup();

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct EmptyTransparentStruct {}

#[derive(IntoPyObject)]
enum EnumWithTransparentEmptyTupleVar {
    #[pyo3(transparent)]
    EmptyTuple(),
    Valid(String),
}

#[derive(IntoPyObject)]
enum EnumWithTransparentEmptyStructVar {
    #[pyo3(transparent)]
    EmptyStruct {},
    Valid(String),
}

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct TransparentTupTooManyFields(String, String);

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct TransparentStructTooManyFields {
    foo: String,
    bar: String,
}

#[derive(IntoPyObject)]
enum EnumWithTransparentTupleTooMany {
    #[pyo3(transparent)]
    EmptyTuple(String, String),
    Valid(String),
}

#[derive(IntoPyObject)]
enum EnumWithTransparentStructTooMany {
    #[pyo3(transparent)]
    EmptyStruct {
        foo: String,
        bar: String,
    },
    Valid(String),
}

#[derive(IntoPyObject)]
#[pyo3(unknown = "should not work")]
struct UnknownContainerAttr {
    a: String,
}

#[derive(IntoPyObject)]
union Union {
    a: usize,
}

#[derive(IntoPyObject)]
enum UnitEnum {
    Unit,
}

#[derive(IntoPyObject)]
struct TupleAttribute(#[pyo3(attribute)] String, usize);

#[derive(IntoPyObject)]
struct TupleItem(#[pyo3(item)] String, usize);

#[derive(IntoPyObject)]
struct StructAttribute {
    #[pyo3(attribute)]
    foo: String,
}

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct StructTransparentItem {
    #[pyo3(item)]
    foo: String,
}

fn main() {}
