## Conversion traits

PyO3 provides some handy traits to convert between Python types and Rust types.

### `.extract()` and the `FromPyObject` trait

The easiest way to convert a Python object to a Rust value is using
`.extract()`.  It returns a `PyResult` with a type error if the conversion
fails, so usually you will use something like

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let list = PyList::new(py, b"foo");
let v: Vec<i32> = list.extract()?;
#         assert_eq!(&v, &[102, 111, 111]);
#         Ok(())
#     })
# }
```

This method is available for many Python object types, and can produce a wide
variety of Rust types, which you can check out in the implementor list of
[`FromPyObject`].

[`FromPyObject`] is also implemented for your own Rust types wrapped as Python
objects (see [the chapter about classes](../class.md)).  There, in order to both be
able to operate on mutable references *and* satisfy Rust's rules of non-aliasing
mutable references, you have to extract the PyO3 reference wrappers [`PyRef`]
and [`PyRefMut`].  They work like the reference wrappers of
`std::cell::RefCell` and ensure (at runtime) that Rust borrows are allowed.

#### Deriving [`FromPyObject`]

[`FromPyObject`] can be automatically derived for many kinds of structs and enums
if the member types themselves implement `FromPyObject`. This even includes members
with a generic type `T: FromPyObject`. Derivation for empty enums, enum variants and
structs is not supported.

#### Deriving [`FromPyObject`] for structs

The derivation generates code that will attempt to access the attribute  `my_string` on
the Python object, i.e. `obj.getattr("my_string")`, and call `extract()` on the attribute.

```rust
use pyo3::prelude::*;
use pyo3_ffi::c_str;

#[derive(FromPyObject)]
struct RustyStruct {
    my_string: String,
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let module = PyModule::from_code(
#             py,
#             c_str!("class Foo:
#             def __init__(self):
#                 self.my_string = 'test'"),
#             c_str!(""),
#             c_str!(""),
#         )?;
#
#         let class = module.getattr("Foo")?;
#         let instance = class.call0()?;
#         let rustystruct: RustyStruct = instance.extract()?;
#         assert_eq!(rustystruct.my_string, "test");
#         Ok(())
#     })
# }
```

By setting the `#[pyo3(item)]` attribute on the field, PyO3 will attempt to extract the value by calling the `get_item` method on the Python object.

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item)]
    my_string: String,
}
#
# use pyo3::types::PyDict;
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let dict = PyDict::new(py);
#         dict.set_item("my_string", "test")?;
#
#         let rustystruct: RustyStruct = dict.extract()?;
#         assert_eq!(rustystruct.my_string, "test");
#         Ok(())
#     })
# }
```

The argument passed to `getattr` and `get_item` can also be configured:

```rust
use pyo3::prelude::*;
use pyo3_ffi::c_str;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item("key"))]
    string_in_mapping: String,
    #[pyo3(attribute("name"))]
    string_attr: String,
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let module = PyModule::from_code(
#             py,
#             c_str!("class Foo(dict):
#             def __init__(self):
#                 self.name = 'test'
#                 self['key'] = 'test2'"),
#             c_str!(""),
#             c_str!(""),
#         )?;
#
#         let class = module.getattr("Foo")?;
#         let instance = class.call0()?;
#         let rustystruct: RustyStruct = instance.extract()?;
# 		assert_eq!(rustystruct.string_attr, "test");
#         assert_eq!(rustystruct.string_in_mapping, "test2");
#
#         Ok(())
#     })
# }
```

This tries to extract `string_attr` from the attribute `name` and `string_in_mapping`
from a mapping with the key `"key"`. The arguments for `attribute` are restricted to
non-empty string literals while `item` can take any valid literal that implements
`ToBorrowedObject`.

You can use `#[pyo3(from_item_all)]` on a struct to extract every field with `get_item` method.
In this case, you can't use `#[pyo3(attribute)]` or barely use `#[pyo3(item)]` on any field.
However, using `#[pyo3(item("key"))]` to specify the key for a field is still allowed.

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
struct RustyStruct {
    foo: String,
    bar: String,
    #[pyo3(item("foobar"))]
    baz: String,
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let py_dict = py.eval_bound("{'foo': 'foo', 'bar': 'bar', 'foobar': 'foobar'}", None, None)?;
#         let rustystruct: RustyStruct = py_dict.extract()?;
# 		  assert_eq!(rustystruct.foo, "foo");
#         assert_eq!(rustystruct.bar, "bar");
#         assert_eq!(rustystruct.baz, "foobar");
#
#         Ok(())
#     })
# }
```

#### Deriving [`FromPyObject`] for tuple structs

Tuple structs are also supported but do not allow customizing the extraction. The input is
always assumed to be a Python tuple with the same length as the Rust type, the `n`th field
is extracted from the `n`th item in the Python tuple.

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple(String, String);

# use pyo3::types::PyTuple;
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let tuple = PyTuple::new(py, vec!["test", "test2"]);
#
#         let rustytuple: RustyTuple = tuple.extract()?;
#         assert_eq!(rustytuple.0, "test");
#         assert_eq!(rustytuple.1, "test2");
#
#         Ok(())
#     })
# }
```

Tuple structs with a single field are treated as wrapper types which are described in the
following section. To override this behaviour and ensure that the input is in fact a tuple,
specify the struct as
```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple((String,));

# use pyo3::types::PyTuple;
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let tuple = PyTuple::new(py, vec!["test"]);
#
#         let rustytuple: RustyTuple = tuple.extract()?;
#         assert_eq!((rustytuple.0).0, "test");
#
#         Ok(())
#     })
# }
```

#### Deriving [`FromPyObject`] for wrapper types

The `pyo3(transparent)` attribute can be used on structs with exactly one field. This results
in extracting directly from the input object, i.e. `obj.extract()`, rather than trying to access
an item or attribute. This behaviour is enabled per default for newtype structs and tuple-variants
with a single field.

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTransparentTupleStruct(String);

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct RustyTransparentStruct {
    inner: String,
}

# use pyo3::types::PyString;
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         let s = PyString::new(py, "test");
#
#         let tup: RustyTransparentTupleStruct = s.extract()?;
#         assert_eq!(tup.0, "test");
#
#         let stru: RustyTransparentStruct = s.extract()?;
#         assert_eq!(stru.inner, "test");
#
#         Ok(())
#     })
# }
```

#### Deriving [`FromPyObject`] for enums

The `FromPyObject` derivation for enums generates code that tries to extract the variants in the
order of the fields. As soon as a variant can be extracted successfully, that variant is returned.
This makes it possible to extract Python union types like `str | int`.

The same customizations and restrictions described for struct derivations apply to enum variants,
i.e. a tuple variant assumes that the input is a Python tuple, and a struct variant defaults to
extracting fields as attributes but can be configured in the same manner. The `transparent`
attribute can be applied to single-field-variants.

```rust
use pyo3::prelude::*;
use pyo3_ffi::c_str;

#[derive(FromPyObject)]
# #[derive(Debug)]
enum RustyEnum<'py> {
    Int(usize),                    // input is a positive int
    String(String),                // input is a string
    IntTuple(usize, usize),        // input is a 2-tuple with positive ints
    StringIntTuple(String, usize), // input is a 2-tuple with String and int
    Coordinates3d {
        // needs to be in front of 2d
        x: usize,
        y: usize,
        z: usize,
    },
    Coordinates2d {
        // only gets checked if the input did not have `z`
        #[pyo3(attribute("x"))]
        a: usize,
        #[pyo3(attribute("y"))]
        b: usize,
    },
    #[pyo3(transparent)]
    CatchAll(Bound<'py, PyAny>), // This extraction never fails
}
#
# use pyo3::types::{PyBytes, PyString};
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         {
#             let thing = 42_u8.to_object(py);
#             let rust_thing: RustyEnum<'_> = thing.extract(py)?;
#
#             assert_eq!(
#                 42,
#                 match rust_thing {
#                     RustyEnum::Int(i) => i,
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#         {
#             let thing = PyString::new(py, "text");
#             let rust_thing: RustyEnum<'_> = thing.extract()?;
#
#             assert_eq!(
#                 "text",
#                 match rust_thing {
#                     RustyEnum::String(i) => i,
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#         {
#             let thing = (32_u8, 73_u8).to_object(py);
#             let rust_thing: RustyEnum<'_> = thing.extract(py)?;
#
#             assert_eq!(
#                 (32, 73),
#                 match rust_thing {
#                     RustyEnum::IntTuple(i, j) => (i, j),
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#         {
#             let thing = ("foo", 73_u8).to_object(py);
#             let rust_thing: RustyEnum<'_> = thing.extract(py)?;
#
#             assert_eq!(
#                 (String::from("foo"), 73),
#                 match rust_thing {
#                     RustyEnum::StringIntTuple(i, j) => (i, j),
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#         {
#             let module = PyModule::from_code(
#                 py,
#                 c_str!("class Foo(dict):
#             def __init__(self):
#                 self.x = 0
#                 self.y = 1
#                 self.z = 2"),
#                 c_str!(""),
#                 c_str!(""),
#             )?;
#
#             let class = module.getattr("Foo")?;
#             let instance = class.call0()?;
#             let rust_thing: RustyEnum<'_> = instance.extract()?;
#
#             assert_eq!(
#                 (0, 1, 2),
#                 match rust_thing {
#                     RustyEnum::Coordinates3d { x, y, z } => (x, y, z),
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#
#         {
#             let module = PyModule::from_code(
#                 py,
#                 c_str!("class Foo(dict):
#             def __init__(self):
#                 self.x = 3
#                 self.y = 4"),
#                 c_str!(""),
#                 c_str!(""),
#             )?;
#
#             let class = module.getattr("Foo")?;
#             let instance = class.call0()?;
#             let rust_thing: RustyEnum<'_> = instance.extract()?;
#
#             assert_eq!(
#                 (3, 4),
#                 match rust_thing {
#                     RustyEnum::Coordinates2d { a, b } => (a, b),
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#
#         {
#             let thing = PyBytes::new(py, b"text");
#             let rust_thing: RustyEnum<'_> = thing.extract()?;
#
#             assert_eq!(
#                 b"text",
#                 match rust_thing {
#                     RustyEnum::CatchAll(ref i) => i.downcast::<PyBytes>()?.as_bytes(),
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#         Ok(())
#     })
# }
```

If none of the enum variants match, a `PyTypeError` containing the names of the
tested variants is returned. The names reported in the error message can be customized
through the `#[pyo3(annotation = "name")]` attribute, e.g. to use conventional Python type
names:

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
# #[derive(Debug)]
enum RustyEnum {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "int")]
    Int(isize),
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| -> PyResult<()> {
#         {
#             let thing = 42_u8.to_object(py);
#             let rust_thing: RustyEnum = thing.extract(py)?;
#
#             assert_eq!(
#                 42,
#                 match rust_thing {
#                     RustyEnum::Int(i) => i,
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#
#         {
#             let thing = "foo".to_object(py);
#             let rust_thing: RustyEnum = thing.extract(py)?;
#
#             assert_eq!(
#                 "foo",
#                 match rust_thing {
#                     RustyEnum::String(i) => i,
#                     other => unreachable!("Error extracting: {:?}", other),
#                 }
#             );
#         }
#
#         {
#             let thing = b"foo".to_object(py);
#             let error = thing.extract::<RustyEnum>(py).unwrap_err();
#             assert!(error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
#         }
#
#         Ok(())
#     })
# }
```

If the input is neither a string nor an integer, the error message will be:
`"'<INPUT_TYPE>' cannot be converted to 'str | int'"`.

#### `#[derive(FromPyObject)]` Container Attributes
- `pyo3(transparent)`
    - extract the field directly from the object as `obj.extract()` instead of `get_item()` or
      `getattr()`
    - Newtype structs and tuple-variants are treated as transparent per default.
    - only supported for single-field structs and enum variants
- `pyo3(annotation = "name")`
    - changes the name of the failed variant in the generated error message in case of failure.
    - e.g. `pyo3("int")` reports the variant's type as `int`.
    - only supported for enum variants

#### `#[derive(FromPyObject)]` Field Attributes
- `pyo3(attribute)`, `pyo3(attribute("name"))`
    - retrieve the field from an attribute, possibly with a custom name specified as an argument
    - argument must be a string-literal.
- `pyo3(item)`, `pyo3(item("key"))`
    - retrieve the field from a mapping, possibly with the custom key specified as an argument.
    - can be any literal that implements `ToBorrowedObject`
- `pyo3(from_py_with = "...")`
    - apply a custom function to convert the field from Python the desired Rust type.
    - the argument must be the name of the function as a string.
    - the function signature must be `fn(&Bound<PyAny>) -> PyResult<T>` where `T` is the Rust type of the argument.

### `IntoPy<T>`

This trait defines the to-python conversion for a Rust type. It is usually implemented as
`IntoPy<PyObject>`, which is the trait needed for returning a value from `#[pyfunction]` and
`#[pymethods]`.

All types in PyO3 implement this trait, as does a `#[pyclass]` which doesn't use `extends`.

Occasionally you may choose to implement this for custom types which are mapped to Python types
_without_ having a unique python type.

```rust
use pyo3::prelude::*;
# #[allow(dead_code)]
struct MyPyObjectWrapper(PyObject);

impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0
    }
}
```

### The `ToPyObject` trait

[`ToPyObject`] is a conversion trait that allows various objects to be
converted into [`PyObject`]. `IntoPy<PyObject>` serves the
same purpose, except that it consumes `self`.

[`IntoPy`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.IntoPy.html
[`FromPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.FromPyObject.html
[`ToPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.ToPyObject.html
[`PyObject`]: {{#PYO3_DOCS_URL}}/pyo3/type.PyObject.html

[`PyRef`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRefMut.html
