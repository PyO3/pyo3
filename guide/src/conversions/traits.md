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
#     Python::attach(|py| {
#         let list = PyList::new(py, b"foo")?;
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
#     Python::attach(|py| -> PyResult<()> {
#         let module = PyModule::from_code(
#             py,
#             c_str!("class Foo:
#             def __init__(self):
#                 self.my_string = 'test'"),
#             c_str!("<string>"),
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
#     Python::attach(|py| -> PyResult<()> {
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
#     Python::attach(|py| -> PyResult<()> {
#         let module = PyModule::from_code(
#             py,
#             c_str!("class Foo(dict):
#             def __init__(self):
#                 self.name = 'test'
#                 self['key'] = 'test2'"),
#             c_str!("<string>"),
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
#     Python::attach(|py| -> PyResult<()> {
#         let py_dict = py.eval(pyo3::ffi::c_str!("{'foo': 'foo', 'bar': 'bar', 'foobar': 'foobar'}"), None, None)?;
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
#     Python::attach(|py| -> PyResult<()> {
#         let tuple = PyTuple::new(py, vec!["test", "test2"])?;
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
#     Python::attach(|py| -> PyResult<()> {
#         let tuple = PyTuple::new(py, vec!["test"])?;
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
#     Python::attach(|py| -> PyResult<()> {
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
#     Python::attach(|py| -> PyResult<()> {
#         {
#             let thing = 42_u8.into_pyobject(py)?;
#             let rust_thing: RustyEnum<'_> = thing.extract()?;
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
#             let thing = (32_u8, 73_u8).into_pyobject(py)?;
#             let rust_thing: RustyEnum<'_> = thing.extract()?;
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
#             let thing = ("foo", 73_u8).into_pyobject(py)?;
#             let rust_thing: RustyEnum<'_> = thing.extract()?;
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
#                 c_str!("<string>"),
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
#                 c_str!("<string>"),
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
#                     RustyEnum::CatchAll(ref i) => i.cast::<PyBytes>()?.as_bytes(),
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
#     Python::attach(|py| -> PyResult<()> {
#         {
#             let thing = 42_u8.into_pyobject(py)?;
#             let rust_thing: RustyEnum = thing.extract()?;
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
#             let thing = "foo".into_pyobject(py)?;
#             let rust_thing: RustyEnum = thing.extract()?;
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
#             let thing = b"foo".into_pyobject(py)?;
#             let error = thing.extract::<RustyEnum>().unwrap_err();
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
- `pyo3(rename_all = "...")`
    - renames all attributes/item keys according to the specified renaming rule
    - Possible values are: "camelCase", "kebab-case", "lowercase", "PascalCase", "SCREAMING-KEBAB-CASE", "SCREAMING_SNAKE_CASE", "snake_case", "UPPERCASE".
    - fields with an explicit renaming via `attribute(...)`/`item(...)` are not affected

#### `#[derive(FromPyObject)]` Field Attributes
- `pyo3(attribute)`, `pyo3(attribute("name"))`
    - retrieve the field from an attribute, possibly with a custom name specified as an argument
    - argument must be a string-literal.
- `pyo3(item)`, `pyo3(item("key"))`
    - retrieve the field from a mapping, possibly with the custom key specified as an argument.
    - can be any literal that implements `ToBorrowedObject`
- `pyo3(from_py_with = ...)`
    - apply a custom function to convert the field from Python the desired Rust type.
    - the argument must be the path to the function.
    - the function signature must be `fn(&Bound<PyAny>) -> PyResult<T>` where `T` is the Rust type of the argument.
- `pyo3(default)`, `pyo3(default = ...)`
  - if the argument is set, uses the given default value.
  - in this case, the argument must be a Rust expression returning a value of the desired Rust type.
  - if the argument is not set, [`Default::default`](https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default) is used.
  - note that the default value is only used if the field is not set.
    If the field is set and the conversion function from Python to Rust fails, an exception is raised and the default value is not used.
  - this attribute is only supported on named fields.

For example, the code below applies the given conversion function on the `"value"` dict item to compute its length or fall back to the type default value (0):

```rust
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item("value"), default, from_py_with = Bound::<'_, PyAny>::len)]
    len: usize,
    #[pyo3(item)]
    other: usize,
}
#
# use pyo3::types::PyDict;
# fn main() -> PyResult<()> {
#     Python::attach(|py| -> PyResult<()> {
#         // Filled case
#         let dict = PyDict::new(py);
#         dict.set_item("value", (1,)).unwrap();
#         dict.set_item("other", 1).unwrap();
#         let result = dict.extract::<RustyStruct>()?;
#         assert_eq!(result.len, 1);
#         assert_eq!(result.other, 1);
#
#         // Empty case
#         let dict = PyDict::new(py);
#         dict.set_item("other", 1).unwrap();
#         let result = dict.extract::<RustyStruct>()?;
#         assert_eq!(result.len, 0);
#         assert_eq!(result.other, 1);
#         Ok(())
#     })
# }
```

### `IntoPyObject`
The [`IntoPyObject`] trait defines the to-python conversion for a Rust type. All types in PyO3 implement this trait,
as does a `#[pyclass]` which doesn't use `extends`.

This trait defines a single method, `into_pyobject()`, which returns a [`Result`] with `Ok` and `Err` types depending on the input value. For convenience, there is a companion [`IntoPyObjectExt`] trait which adds methods such as `into_py_any()` which converts the `Ok` and `Err` types to commonly used types (in the case of `into_py_any()`, `Py<PyAny>` and `PyErr` respectively).

Occasionally you may choose to implement this for custom types which are mapped to Python types
_without_ having a unique python type.

#### derive macro

`IntoPyObject` can be implemented using our derive macro. Both `struct`s and `enum`s are supported.

`struct`s will turn into a `PyDict` using the field names as keys, tuple `struct`s will turn convert
into `PyTuple` with the fields in declaration order.
```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use std::collections::HashMap;
# use std::hash::Hash;

// structs convert into `PyDict` with field names as keys
#[derive(IntoPyObject)]
struct Struct {
    count: usize,
    obj: Py<PyAny>,
}

// tuple structs convert into `PyTuple`
// lifetimes and generics are supported, the impl will be bounded by
// `K: IntoPyObject, V: IntoPyObject`
#[derive(IntoPyObject)]
struct Tuple<'a, K: Hash + Eq, V>(&'a str, HashMap<K, V>);
```

For structs with a single field (newtype pattern) the `#[pyo3(transparent)]` option can be used to
forward the implementation to the inner type.


```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;

// newtype tuple structs are implicitly `transparent`
#[derive(IntoPyObject)]
struct TransparentTuple(Py<PyAny>);

#[derive(IntoPyObject)]
#[pyo3(transparent)]
struct TransparentStruct<'py> {
    inner: Bound<'py, PyAny>, // `'py` lifetime will be used as the Python lifetime
}
```

For `enum`s each variant is converted according to the rules for `struct`s above.

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use std::collections::HashMap;
# use std::hash::Hash;

#[derive(IntoPyObject)]
enum Enum<'a, 'py, K: Hash + Eq, V> { // enums are supported and convert using the same
    TransparentTuple(Py<PyAny>),       // rules on the variants as the structs above
    #[pyo3(transparent)]
    TransparentStruct { inner: Bound<'py, PyAny> },
    Tuple(&'a str, HashMap<K, V>),
    Struct { count: usize, obj: Py<PyAny> }
}
```

Additionally `IntoPyObject` can be derived for a reference to a struct or enum using the
`IntoPyObjectRef` derive macro. All the same rules from above apply as well.

##### `#[derive(IntoPyObject)]`/`#[derive(IntoPyObjectRef)]` Field Attributes
- `pyo3(into_py_with = ...)`
    - apply a custom function to convert the field from Rust into Python.
    - the argument must be the function identifier
    - the function signature must be `fn(Cow<'_, T>, Python<'py>) -> PyResult<Bound<'py, PyAny>>` where `T` is the Rust type of the argument.
      - `#[derive(IntoPyObject)]` will invoke the function with `Cow::Owned`
      - `#[derive(IntoPyObjectRef)]` will invoke the function with `Cow::Borrowed`

    ```rust,no_run
    # use pyo3::prelude::*;
    # use pyo3::IntoPyObjectExt;
    # use std::borrow::Cow;
    #[derive(Clone)]
    struct NotIntoPy(usize);

    #[derive(IntoPyObject, IntoPyObjectRef)]
    struct MyStruct {
        #[pyo3(into_py_with = convert)]
        not_into_py: NotIntoPy,
    }

    /// Convert `NotIntoPy` into Python
    fn convert<'py>(not_into_py: Cow<'_, NotIntoPy>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        not_into_py.0.into_bound_py_any(py)
    }
    ```

#### manual implementation

If the derive macro is not suitable for your use case, `IntoPyObject` can be implemented manually as
demonstrated below.

```rust,no_run
# use pyo3::prelude::*;
# #[allow(dead_code)]
struct MyPyObjectWrapper(Py<PyAny>);

impl<'py> IntoPyObject<'py> for MyPyObjectWrapper {
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
    type Error = std::convert::Infallible; // the conversion error type, has to be convertible to `PyErr`

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

// equivalent to former `ToPyObject` implementations
impl<'a, 'py> IntoPyObject<'py> for &'a MyPyObjectWrapper {
    type Target = PyAny;
    type Output = Borrowed<'a, 'py, Self::Target>; // `Borrowed` can be used to optimized reference counting
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.bind_borrowed(py))
    }
}
```

#### `BoundObject` for conversions that may be `Bound` or `Borrowed`

`IntoPyObject::into_py_object` returns either `Bound` or `Borrowed` depending on the implementation for a concrete type. For example, the `IntoPyObject` implementation for `u32` produces a `Bound<'py, PyInt>` and the `bool` implementation produces a `Borrowed<'py, 'py, PyBool>`:

```rust,no_run
use pyo3::prelude::*;
use pyo3::IntoPyObject;
use pyo3::types::{PyBool, PyInt};

let ints: Vec<u32> = vec![1, 2, 3, 4];
let bools = vec![true, false, false, true];

Python::attach(|py| {
    let ints_as_pyint: Vec<Bound<'_, PyInt>> = ints
        .iter()
        .map(|x| Ok(x.into_pyobject(py)?))
        .collect::<PyResult<_>>()
        .unwrap();

    let bools_as_pybool: Vec<Borrowed<'_, '_, PyBool>> = bools
        .iter()
        .map(|x| Ok(x.into_pyobject(py)?))
        .collect::<PyResult<_>>()
        .unwrap();
});
```

In this example if we wanted to combine `ints_as_pyints` and `bools_as_pybool` into a single `Vec<Py<PyAny>>` to return from the `Python::attach` closure, we would have to manually convert the concrete types for the smart pointers and the python types.

Instead, we can write a function that generically converts vectors of either integers or bools into a vector of `Py<PyAny>` using the [`BoundObject`] trait:

```rust,no_run
# use pyo3::prelude::*;
# use pyo3::BoundObject;
# use pyo3::IntoPyObject;

# let bools = vec![true, false, false, true];
# let ints = vec![1, 2, 3, 4];

fn convert_to_vec_of_pyobj<'py, T>(py: Python<'py>, the_vec: Vec<T>) -> PyResult<Vec<Py<PyAny>>>
where
   T: IntoPyObject<'py> + Copy
{
    the_vec.iter()
        .map(|x| {
            Ok(
                // Note: the below is equivalent to `x.into_py_any()`
                // from the `IntoPyObjectExt` trait
                x.into_pyobject(py)
                .map_err(Into::into)?
                .into_any()
                .unbind()
            )
        })
        .collect()
}

let vec_of_pyobjs: Vec<Py<PyAny>> = Python::attach(|py| {
    let mut bools_as_pyany = convert_to_vec_of_pyobj(py, bools).unwrap();
    let mut ints_as_pyany = convert_to_vec_of_pyobj(py, ints).unwrap();
    let mut result: Vec<Py<PyAny>> = vec![];
    result.append(&mut bools_as_pyany);
    result.append(&mut ints_as_pyany);
    result
});
```

In the example above we used `BoundObject::into_any` and `BoundObject::unbind` to manipulate the python types and smart pointers into the result type we wanted to produce from the function.

[`FromPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.FromPyObject.html
[`IntoPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.IntoPyObject.html
[`IntoPyObjectExt`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.IntoPyObjectExt.html

[`PyRef`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRefMut.html
[`BoundObject`]: {{#PYO3_DOCS_URL}}/pyo3/instance/trait.BoundObject.html

[`Result`]: https://doc.rust-lang.org/stable/std/result/enum.Result.html
