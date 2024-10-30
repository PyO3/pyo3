use pyo3::{IntoPyObject, IntoPyObjectRef};

#[derive(IntoPyObject, IntoPyObjectRef)]
struct Foo();

#[derive(IntoPyObject, IntoPyObjectRef)]
struct Foo2 {}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EmptyEnum {}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithEmptyTupleVar {
    EmptyTuple(),
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithEmptyStructVar {
    EmptyStruct {},
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct EmptyTransparentTup();

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct EmptyTransparentStruct {}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithTransparentEmptyTupleVar {
    #[pyo3(transparent)]
    EmptyTuple(),
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithTransparentEmptyStructVar {
    #[pyo3(transparent)]
    EmptyStruct {},
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct TransparentTupTooManyFields(String, String);

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct TransparentStructTooManyFields {
    foo: String,
    bar: String,
}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithTransparentTupleTooMany {
    #[pyo3(transparent)]
    EmptyTuple(String, String),
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum EnumWithTransparentStructTooMany {
    #[pyo3(transparent)]
    EmptyStruct {
        foo: String,
        bar: String,
    },
    Valid(String),
}

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(unknown = "should not work")]
struct UnknownContainerAttr {
    a: String,
}

#[derive(IntoPyObject, IntoPyObjectRef)]
union Union {
    a: usize,
}

#[derive(IntoPyObject, IntoPyObjectRef)]
enum UnitEnum {
    Unit,
}

#[derive(IntoPyObject, IntoPyObjectRef)]
struct TupleAttribute(#[pyo3(attribute)] String, usize);

#[derive(IntoPyObject, IntoPyObjectRef)]
struct TupleItem(#[pyo3(item)] String, usize);

#[derive(IntoPyObject, IntoPyObjectRef)]
struct StructAttribute {
    #[pyo3(attribute)]
    foo: String,
}

#[derive(IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct StructTransparentItem {
    #[pyo3(item)]
    foo: String,
}

fn main() {}
