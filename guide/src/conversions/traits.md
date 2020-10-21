## Conversion traits

PyO3 provides some handy traits to convert between Python types and Rust types.

### `.extract()` and the `FromPyObject` trait

The easiest way to convert a Python object to a Rust value is using
`.extract()`.  It returns a `PyResult` with a type error if the conversion
fails, so usually you will use something like

```ignore
let v: Vec<i32> = obj.extract()?;
```

This method is available for many Python object types, and can produce a wide
variety of Rust types, which you can check out in the implementor list of
[`FromPyObject`].

[`FromPyObject`] is also implemented for your own Rust types wrapped as Python
objects (see [the chapter about classes](class.md)).  There, in order to both be
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

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    my_string: String,
}
```

The derivation generates code that will per default access the attribute `my_string` on
the Python object, i.e. `obj.getattr("my_string")`, and call `extract()` on the attribute.
It is also possible to access the value on the Python object through `obj.get_item("my_string")`
by setting the attribute `pyo3(item)` on the field:
```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item)]
    my_string: String,
}
```

The argument passed to `getattr` and `get_item` can also be configured:

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item("key"))]
    string_in_mapping: String,
    #[pyo3(attribute("name"))]
    string_attr: String,
}
```

This tries to extract `string_attr` from the attribute `name` and `string_in_mapping`
from a mapping with the key `"key"`. The arguments for `attribute` are restricted to
non-empty string literals while `item` can take any valid literal that implements
`ToBorrowedObject`.

#### Deriving [`FromPyObject`] for tuple structs

Tuple structs are also supported but do not allow customizing the extraction. The input is
always assumed to be a Python tuple with the same length as the Rust type, the `n`th field
is extracted from the `n`th item in the Python tuple.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple(String, String);
```

Tuple structs with a single field are treated as wrapper types which are described in the
following section. To override this behaviour and ensure that the input is in fact a tuple,
specify the struct as
```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple((String,));
```

#### Deriving [`FromPyObject`] for wrapper types

The `pyo3(transparent)` attribute can be used on structs with exactly one field. This results
in extracting directly from the input object, i.e. `obj.extract()`, rather than trying to access
an item or attribute. This behaviour is enabled per default for newtype structs and tuple-variants
with a single field.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTransparentTupleStruct(String);

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct RustyTransparentStruct {
    inner: String,
}
```

#### Deriving [`FromPyObject`] for enums

The `FromPyObject` derivation for enums generates code that tries to extract the variants in the
order of the fields. As soon as a variant can be extracted succesfully, that variant is returned.
This makes it possible to extract Python types like `Union[str, int]`.

The same customizations and restrictions described for struct derivations apply to enum variants,
i.e. a tuple variant assumes that the input is a Python tuple, and a struct variant defaults to
extracting fields as attributes but can be configured in the same manner. The `transparent`
attribute can be applied to single-field-variants.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
enum RustyEnum<'a> {
    Int(usize), // input is a positive int
    String(String), // input is a string
    IntTuple(usize, usize), // input is a 2-tuple with positive ints
    StringIntTuple(String, usize), // input is a 2-tuple with String and int
    Coordinates3d { // needs to be in front of 2d
        x: usize,
        y: usize,
        z: usize,
    },
    Coordinates2d { // only gets checked if the input did not have `z`
        #[pyo3(attribute("x"))]
        a: usize,
        #[pyo3(attribute("y"))]
        b: usize,
    },
    #[pyo3(transparent)]
    CatchAll(&'a PyAny), // This extraction never fails
}
```

If none of the enum variants match, a `PyValueError` containing the names of the
tested variants is returned. The names reported in the error message can be customized
through the `pyo3(annotation = "name")` attribute, e.g. to use conventional Python type
names:

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
enum RustyEnum {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "int")]
    Int(isize),
}
```

If the input is neither a string nor an integer, the error message will be:
`"'<INPUT_TYPE>' cannot be converted to 'Union[str, int]'"`.

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

### `IntoPy<T>`

This trait defines the to-python conversion for a Rust type. It is usually implemented as
`IntoPy<PyObject>`, which is the trait needed for returning a value from `#[pyfunction]` and
`#[pymethods]`.

All types in PyO3 implement this trait, as does a `#[pyclass]` which doesn't use `extends`.

Occasionally you may choose to implement this for custom types which are mapped to Python types
_without_ having a unique python type.

```
use pyo3::prelude::*;

struct MyPyObjectWrapper(PyObject);

impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, py: Python) -> PyObject {
        self.0
    }
}
```

### The `ToPyObject` trait

[`ToPyObject`] is a conversion trait that allows various objects to be
converted into [`PyObject`]. `IntoPy<PyObject>` serves the
same purpose, except that it consumes `self`.

[`IntoPy`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.IntoPy.html
[`FromPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html
[`ToPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.ToPyObject.html
[`PyObject`]: https://docs.rs/pyo3/latest/pyo3/type.PyObject.html

[`PyRef`]: https://docs.rs/pyo3/latest/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: https://docs.rs/pyo3/latest/pyo3/pycell/struct.PyRefMut.html
