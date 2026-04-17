use pyo3::FromPyObject;

#[derive(FromPyObject)]
struct Foo();
//~^ ERROR: cannot derive FromPyObject for empty structs and variants

#[derive(FromPyObject)]
struct Foo2 {}
//~^ ERROR: cannot derive FromPyObject for empty structs and variants

#[derive(FromPyObject)]
enum EmptyEnum {}
//~^ ERROR: cannot derive FromPyObject for empty enum

#[derive(FromPyObject)]
enum EnumWithEmptyTupleVar {
    EmptyTuple(),
    //~^ ERROR: cannot derive FromPyObject for empty structs and variants
    Valid(String),
}

#[derive(FromPyObject)]
enum EnumWithEmptyStructVar {
    EmptyStruct {},
    //~^ ERROR: cannot derive FromPyObject for empty structs and variants
    Valid(String),
}

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct EmptyTransparentTup();
//~^ ERROR: cannot derive FromPyObject for empty structs and variants

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct EmptyTransparentStruct {}
//~^ ERROR: cannot derive FromPyObject for empty structs and variants

#[derive(FromPyObject)]
enum EnumWithTransparentEmptyTupleVar {
    #[pyo3(transparent)]
    EmptyTuple(),
    //~^ ERROR: cannot derive FromPyObject for empty structs and variants
    Valid(String),
}

#[derive(FromPyObject)]
enum EnumWithTransparentEmptyStructVar {
    #[pyo3(transparent)]
    EmptyStruct {},
    //~^ ERROR: cannot derive FromPyObject for empty structs and variants
    Valid(String),
}

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct TransparentTupTooManyFields(String, String);
//~^ ERROR: transparent structs and variants can only have 1 field

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct TransparentStructTooManyFields {
    //~^ ERROR: transparent structs and variants can only have 1 field
    foo: String,
    bar: String,
}

#[derive(FromPyObject)]
enum EnumWithTransparentTupleTooMany {
    #[pyo3(transparent)]
    EmptyTuple(String, String),
    //~^ ERROR: transparent structs and variants can only have 1 field
    Valid(String),
}

#[derive(FromPyObject)]
enum EnumWithTransparentStructTooMany {
    #[pyo3(transparent)]
    EmptyStruct {
        //~^ ERROR: transparent structs and variants can only have 1 field
        foo: String,
        bar: String,
    },
    Valid(String),
}

#[derive(FromPyObject)]
struct UnknownAttribute {
    #[pyo3(attr)]
    //~^ ERROR: expected one of: `attribute`, `item`, `from_py_with`, `into_py_with`, `default`
    a: String,
}

#[derive(FromPyObject)]
struct InvalidAttributeArg {
    #[pyo3(attribute(1))]
    //~^ ERROR: expected string literal
    a: String,
}

#[derive(FromPyObject)]
struct TooManyAttributeArgs {
    #[pyo3(attribute("a", "b"))]
    //~^ ERROR: expected at most one argument: `attribute` or `attribute("name")`
    a: String,
}

#[derive(FromPyObject)]
struct EmptyAttributeArg {
    #[pyo3(attribute(""))]
    //~^ ERROR: attribute name cannot be empty
    a: String,
}

#[derive(FromPyObject)]
struct NoAttributeArg {
    #[pyo3(attribute())]
    //~^ ERROR: unexpected end of input, expected string literal
    a: String,
}

#[derive(FromPyObject)]
struct TooManyitemArgs {
    #[pyo3(item("a", "b"))]
    //~^ ERROR: expected at most one argument: `item` or `item(key)`
    a: String,
}

#[derive(FromPyObject)]
struct NoItemArg {
    #[pyo3(item())]
    //~^ ERROR: unexpected end of input, expected literal
    a: String,
}

#[derive(FromPyObject)]
struct ItemAndAttribute {
    #[pyo3(item, attribute)]
    //~^ ERROR: only one of `attribute` or `item` can be provided
    a: String,
}

#[derive(FromPyObject)]
#[pyo3(unknown = "should not work")]
//~^ ERROR: expected one of: `transparent`, `from_item_all`, `annotation`, `crate`, `rename_all`
struct UnknownContainerAttr {
    a: String,
}

#[derive(FromPyObject)]
#[pyo3(annotation = "should not work")]
//~^ ERROR: `annotation` is unsupported for structs
struct AnnotationOnStruct {
    a: String,
}

#[derive(FromPyObject)]
enum InvalidAnnotatedEnum {
    #[pyo3(annotation = 1)]
    //~^ ERROR: expected string literal
    Foo(String),
}

#[derive(FromPyObject)]
enum TooManyLifetimes<'a, 'b> {
    //~^ ERROR: FromPyObject can be derived with at most one lifetime parameter
    Foo(&'a str),
    Bar(&'b str),
}

#[derive(FromPyObject)]
union Union {
    //~^ ERROR: #[derive(FromPyObject)] is not supported for unions
    a: usize,
}

#[derive(FromPyObject)]
//~^ ERROR: cannot derive FromPyObject for empty structs and variants
enum UnitEnum {
    Unit,
}

#[derive(FromPyObject)]
struct InvalidFromPyWith {
    #[pyo3(from_py_with)]
    //~^ ERROR: expected `=`
    field: String,
}

#[derive(FromPyObject)]
struct InvalidFromPyWithNotFound {
    #[pyo3(from_py_with = func)]
    field: String,
}

#[derive(FromPyObject)]
struct InvalidTupleGetter(#[pyo3(item("foo"))] String);
//~^ ERROR: `getter` is not permitted on tuple struct elements.

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct InvalidTransparentWithGetter {
    #[pyo3(item("foo"))]
    field: String,
    //~^ ERROR: `transparent` structs may not have a `getter` for the inner field
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
struct FromItemAllOnTuple(String);

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
#[pyo3(transparent)]
struct FromItemAllWithTransparent {
    field: String,
    //~^ ERROR: `transparent` structs may not have a `getter` for the inner field
}

#[derive(FromPyObject)]
#[pyo3(from_item_all, from_item_all)]
//~^ ERROR: `from_item_all` may only be specified once
struct MultipleFromItemAll {
    field: String,
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
//~^ ERROR: Useless `item` - the struct is already annotated with `from_item_all`
struct UselessItemAttr {
    #[pyo3(item)]
    field: String,
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
//~^ ERROR: The struct is already annotated with `from_item_all`, `attribute` is not allowed
struct FromItemAllConflictAttr {
    #[pyo3(attribute)]
    field: String,
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
//~^ ERROR: The struct is already annotated with `from_item_all`, `attribute` is not allowed
struct FromItemAllConflictAttrWithArgs {
    #[pyo3(attribute("f"))]
    field: String,
}

#[derive(FromPyObject)]
struct StructWithOnlyDefaultValues {
    //~^ ERROR: cannot derive FromPyObject for structs and variants with only default values
    #[pyo3(default)]
    field: String,
}

#[derive(FromPyObject)]
enum EnumVariantWithOnlyDefaultValues {
    Foo {
        //~^ ERROR: cannot derive FromPyObject for structs and variants with only default values
        #[pyo3(default)]
        field: String,
    },
}

#[derive(FromPyObject)]
struct NamedTuplesWithDefaultValues(#[pyo3(default)] String);
//~^ ERROR: `default` is not permitted on tuple struct elements.

#[derive(FromPyObject)]
#[pyo3(rename_all = "camelCase", rename_all = "kebab-case")]
//~^ ERROR: `rename_all` may only be specified once
struct MultipleRenames {
    snake_case: String,
}

#[derive(FromPyObject)]
#[pyo3(rename_all = "camelCase")]
//~^ ERROR: `rename_all` is useless on tuple structs and variants.
struct RenameAllTuple(String);

#[derive(FromPyObject)]
enum RenameAllEnum {
    #[pyo3(rename_all = "camelCase")]
    //~^ ERROR: `rename_all` is useless on tuple structs and variants.
    Tuple(String),
}

#[derive(FromPyObject)]
#[pyo3(transparent, rename_all = "camelCase")]
//~^ ERROR: `rename_all` is not permitted on `transparent` structs and variants
struct RenameAllTransparent {
    inner: String,
}

#[derive(FromPyObject)]
#[pyo3(rename_all = "camelCase")]
enum UselessRenameAllEnum {
    #[pyo3(rename_all = "camelCase")]
    //~^ ERROR: Useless variant `rename_all` - enum is already annotated with `rename_all
    Tuple { inner_field: String },
}

fn main() {}
