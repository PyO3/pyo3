//! Runtime inspection of objects exposed to Python.
//!
//! Tracking issue: <https://github.com/PyO3/pyo3/issues/2454>.

use std::fmt::{self, Display, Write};

pub mod types;

/// Builds a type hint from a module name and a member name in the module
///
/// ```
/// use pyo3::type_hint_identifier;
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_identifier!("datetime", "date");
/// assert_eq!(T.to_string(), "datetime.date");
///
/// const T2: PyStaticExpr = type_hint_identifier!("builtins", "int");
/// assert_eq!(T2.to_string(), "int");
/// ```
#[macro_export]
macro_rules! type_hint_identifier {
    ("builtins", $name:expr) => {
        $crate::inspect::PyStaticExpr::Name { id: $name }
    };
    ($module:expr, $name:expr) => {
        $crate::inspect::PyStaticExpr::Attribute {
            value: &$crate::inspect::PyStaticExpr::Name { id: $module },
            attr: $name,
        }
    };
}
pub(crate) use type_hint_identifier;

/// Builds the union of multiple type hints
///
/// ```
/// use pyo3::{type_hint_identifier, type_hint_union};
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_union!(type_hint_identifier!("builtins", "int"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T.to_string(), "int | float");
/// ```
#[macro_export]
macro_rules! type_hint_union {
    ($e:expr) => { $e };
    ($l:expr , $($r:expr),+) => { $crate::inspect::PyStaticExpr::BinOp {
        left: &$l,
        op: $crate::inspect::PyStaticOperator::BitOr,
        right: &type_hint_union!($($r),+),
    } };
}
pub(crate) use type_hint_union;

/// Builds a subscribed type hint
///
/// ```
/// use pyo3::{type_hint_identifier, type_hint_subscript};
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_subscript!(type_hint_identifier!("collections.abc", "Sequence"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T.to_string(), "collections.abc.Sequence[float]");
///
/// const T2: PyStaticExpr = type_hint_subscript!(type_hint_identifier!("builtins", "dict"), type_hint_identifier!("builtins", "str"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T2.to_string(), "dict[str, float]");
/// ```
#[macro_export]
macro_rules! type_hint_subscript {
    ($l:expr, $r:expr) => {
        $crate::inspect::PyStaticExpr::Subscript {
            value: &$l,
            slice: &$r
        }
    };
    ($l:expr, $($r:expr),*) => {
        $crate::inspect::PyStaticExpr::Subscript {
            value: &$l,
            slice: &$crate::inspect::PyStaticExpr::Tuple { elts: &[$($r),*] }
        }
    };
}
pub(crate) use type_hint_subscript;

/// A Python expression.
///
/// This is the `expr` production of the [Python `ast` module grammar](https://docs.python.org/3/library/ast.html#abstract-grammar)
///
/// This struct aims at being used in `const` contexts like in [`FromPyObject::INPUT_TYPE`](crate::FromPyObject::INPUT_TYPE) and [`IntoPyObject::OUTPUT_TYPE`](crate::IntoPyObject::OUTPUT_TYPE).
///
/// Use macros like [`type_hint_identifier`], [`type_hint_union`] and [`type_hint_subscript`] to construct values.
#[derive(Clone, Copy)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum PyStaticExpr {
    /// A constant like `None` or `123`
    Constant { value: PyStaticConstant },
    /// A name
    Name { id: &'static str },
    /// An attribute `value.attr`
    Attribute {
        value: &'static Self,
        attr: &'static str,
    },
    /// A binary operator
    BinOp {
        left: &'static Self,
        op: PyStaticOperator,
        right: &'static Self,
    },
    /// A tuple
    Tuple { elts: &'static [Self] },
    /// A list
    List { elts: &'static [Self] },
    /// A subscript `value[slice]`
    Subscript {
        value: &'static Self,
        slice: &'static Self,
    },
    /// A `#[pyclass]` type. This is separated type for introspection reasons.
    PyClass(PyClassNameStaticExpr),
}

/// Serialize the type for introspection and return the number of written bytes
#[doc(hidden)]
pub const fn serialize_for_introspection(expr: &PyStaticExpr, mut output: &mut [u8]) -> usize {
    let original_len = output.len();
    match expr {
        PyStaticExpr::Constant { value } => match value {
            PyStaticConstant::None => {
                output = write_slice_and_move_forward(
                    b"{\"type\":\"constant\",\"kind\":\"none\"}",
                    output,
                )
            }
            PyStaticConstant::Bool(value) => {
                output = write_slice_and_move_forward(
                    if *value {
                        b"{\"type\":\"constant\",\"kind\":\"bool\",\"value\":true}"
                    } else {
                        b"{\"type\":\"constant\",\"kind\":\"bool\",\"value\":false}"
                    },
                    output,
                )
            }
            PyStaticConstant::Int(value) => {
                output = write_slice_and_move_forward(
                    b"{\"type\":\"constant\",\"kind\":\"int\",\"value\":\"",
                    output,
                );
                output = write_slice_and_move_forward(value.as_bytes(), output);
                output = write_slice_and_move_forward(b"\"}", output);
            }
            PyStaticConstant::Float(value) => {
                output = write_slice_and_move_forward(
                    b"{\"type\":\"constant\",\"kind\":\"float\",\"value\":\"",
                    output,
                );
                output = write_slice_and_move_forward(value.as_bytes(), output);
                output = write_slice_and_move_forward(b"\"}", output);
            }
            PyStaticConstant::Str(value) => {
                output = write_slice_and_move_forward(
                    b"{\"type\":\"constant\",\"kind\":\"str\",\"value\":",
                    output,
                );
                output = write_json_string_and_move_forward(value.as_bytes(), output);
                output = write_slice_and_move_forward(b"}", output);
            }
            PyStaticConstant::Ellipsis => {
                output = write_slice_and_move_forward(
                    b"{\"type\":\"constant\",\"kind\":\"ellipsis\"}",
                    output,
                )
            }
        },
        PyStaticExpr::Name { id } => {
            output = write_slice_and_move_forward(b"{\"type\":\"name\",\"id\":\"", output);
            output = write_slice_and_move_forward(id.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        PyStaticExpr::Attribute { value, attr } => {
            output = write_slice_and_move_forward(b"{\"type\":\"attribute\",\"value\":", output);
            output = write_expr_and_move_forward(value, output);
            output = write_slice_and_move_forward(b",\"attr\":\"", output);
            output = write_slice_and_move_forward(attr.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        PyStaticExpr::BinOp { left, op, right } => {
            output = write_slice_and_move_forward(b"{\"type\":\"binop\",\"left\":", output);
            output = write_expr_and_move_forward(left, output);
            output = write_slice_and_move_forward(b",\"op\":\"", output);
            output = write_slice_and_move_forward(
                match op {
                    PyStaticOperator::BitOr => b"bitor",
                },
                output,
            );
            output = write_slice_and_move_forward(b"\",\"right\":", output);
            output = write_expr_and_move_forward(right, output);
            output = write_slice_and_move_forward(b"}", output);
        }
        PyStaticExpr::Tuple { elts } => {
            output = write_container_and_move_forward(b"tuple", elts, output);
        }
        PyStaticExpr::List { elts } => {
            output = write_container_and_move_forward(b"list", elts, output);
        }
        PyStaticExpr::Subscript { value, slice } => {
            output = write_slice_and_move_forward(b"{\"type\":\"subscript\",\"value\":", output);
            output = write_expr_and_move_forward(value, output);
            output = write_slice_and_move_forward(b",\"slice\":", output);
            output = write_expr_and_move_forward(slice, output);
            output = write_slice_and_move_forward(b"}", output);
        }
        PyStaticExpr::PyClass(expr) => {
            output = write_slice_and_move_forward(b"{\"type\":\"id\",\"id\":\"", output);
            output = write_slice_and_move_forward(expr.introspection_id.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
    }
    original_len - output.len()
}

/// Length required by [`serialize_for_introspection`]
#[doc(hidden)]
pub const fn serialized_len_for_introspection(expr: &PyStaticExpr) -> usize {
    match expr {
        PyStaticExpr::Constant { value } => match value {
            PyStaticConstant::None => 33,
            PyStaticConstant::Bool(value) => 42 + if *value { 4 } else { 5 },
            PyStaticConstant::Int(value) => 43 + value.len(),
            PyStaticConstant::Float(value) => 45 + value.len(),
            PyStaticConstant::Str(value) => 41 + serialized_json_string_len(value),
            PyStaticConstant::Ellipsis => 37,
        },
        PyStaticExpr::Name { id } => 23 + id.len(),
        PyStaticExpr::Attribute { value, attr } => {
            39 + serialized_len_for_introspection(value) + attr.len()
        }
        PyStaticExpr::BinOp { left, op, right } => {
            41 + serialized_len_for_introspection(left)
                + match op {
                    PyStaticOperator::BitOr => 5,
                }
                + serialized_len_for_introspection(right)
        }
        PyStaticExpr::Tuple { elts } => 5 + serialized_container_len_for_introspection(elts),
        PyStaticExpr::List { elts } => 4 + serialized_container_len_for_introspection(elts),
        PyStaticExpr::Subscript { value, slice } => {
            38 + serialized_len_for_introspection(value) + serialized_len_for_introspection(slice)
        }
        PyStaticExpr::PyClass(expr) => 21 + expr.introspection_id.len(),
    }
}

impl fmt::Display for PyStaticExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant { value } => match value {
                PyStaticConstant::None => f.write_str("None"),
                PyStaticConstant::Bool(value) => f.write_str(if *value { "True" } else { "False" }),
                PyStaticConstant::Int(value) => f.write_str(value),
                PyStaticConstant::Float(value) => {
                    f.write_str(value)?;
                    if !value.contains(['.', 'e', 'E']) {
                        // Makes sure it's not parsed as an int
                        f.write_char('.')?;
                    }
                    Ok(())
                }
                PyStaticConstant::Str(value) => write!(f, "{value:?}"),
                PyStaticConstant::Ellipsis => f.write_str("..."),
            },
            Self::Name { id, .. } => f.write_str(id),
            Self::Attribute { value, attr } => {
                value.fmt(f)?;
                f.write_str(".")?;
                f.write_str(attr)
            }
            Self::BinOp { left, op, right } => {
                left.fmt(f)?;
                f.write_char(' ')?;
                f.write_char(match op {
                    PyStaticOperator::BitOr => '|',
                })?;
                f.write_char(' ')?;
                right.fmt(f)
            }
            Self::Tuple { elts } => {
                f.write_char('(')?;
                fmt_elements(elts, f)?;
                if elts.len() == 1 {
                    f.write_char(',')?;
                }
                f.write_char(')')
            }
            Self::List { elts } => {
                f.write_char('[')?;
                fmt_elements(elts, f)?;
                f.write_char(']')
            }
            Self::Subscript { value, slice } => {
                value.fmt(f)?;
                f.write_char('[')?;
                if let PyStaticExpr::Tuple { elts } = slice {
                    // We don't display the tuple parentheses
                    fmt_elements(elts, f)?;
                } else {
                    slice.fmt(f)?;
                }
                f.write_char(']')
            }
            Self::PyClass(expr) => expr.expr.fmt(f),
        }
    }
}

/// A PyO3 extension to the Python AST to know more about [`PyStaticExpr::Constant`].
///
/// This enables advanced features like escaping.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum PyStaticConstant {
    /// None
    None,
    /// The `True` and `False` booleans
    Bool(bool),
    /// `int` value written in base 10 (`[+-]?[0-9]+`)
    Int(&'static str),
    /// `float` value written in base-10 (`[+-]?[0-9]*(.[0-9]*)*([eE])[0-9]*`), not including Inf and NaN
    Float(&'static str),
    /// `str` value unescaped and without quotes
    Str(&'static str),
    /// `...` value
    Ellipsis,
}

/// An operator used in [`PyStaticExpr::BinOp`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum PyStaticOperator {
    /// `|` operator
    BitOr,
}

const fn write_slice_and_move_forward<'a>(value: &[u8], output: &'a mut [u8]) -> &'a mut [u8] {
    // TODO: use copy_from_slice with MSRV 1.87+
    let mut i = 0;
    while i < value.len() {
        output[i] = value[i];
        i += 1;
    }
    output.split_at_mut(value.len()).1
}

const fn write_json_string_and_move_forward<'a>(
    value: &[u8],
    output: &'a mut [u8],
) -> &'a mut [u8] {
    let mut input_i = 0;
    let mut output_i = 0;
    output[output_i] = b'"';
    output_i += 1;
    while input_i < value.len() {
        match value[input_i] {
            b'\\' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'\\';
                output_i += 1;
            }
            b'"' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'"';
                output_i += 1;
            }
            0x08 => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'b';
                output_i += 1;
            }
            0x0C => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'f';
                output_i += 1;
            }
            b'\n' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'n';
                output_i += 1;
            }
            b'\r' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'r';
                output_i += 1;
            }
            b'\t' => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b't';
                output_i += 1;
            }
            c @ 0..32 => {
                output[output_i] = b'\\';
                output_i += 1;
                output[output_i] = b'u';
                output_i += 1;
                output[output_i] = b'0';
                output_i += 1;
                output[output_i] = b'0';
                output_i += 1;
                output[output_i] = b'0' + (c / 16);
                output_i += 1;
                let remainer = c % 16;
                output[output_i] = if remainer >= 10 {
                    b'A' + remainer - 10
                } else {
                    b'0' + remainer
                };
                output_i += 1;
            }
            c => {
                output[output_i] = c;
                output_i += 1;
            }
        }
        input_i += 1;
    }
    output[output_i] = b'"';
    output_i += 1;
    output.split_at_mut(output_i).1
}

const fn write_expr_and_move_forward<'a>(
    value: &PyStaticExpr,
    output: &'a mut [u8],
) -> &'a mut [u8] {
    let written = serialize_for_introspection(value, output);
    output.split_at_mut(written).1
}

const fn write_container_and_move_forward<'a>(
    name: &'static [u8],
    elts: &[PyStaticExpr],
    mut output: &'a mut [u8],
) -> &'a mut [u8] {
    output = write_slice_and_move_forward(b"{\"type\":\"", output);
    output = write_slice_and_move_forward(name, output);
    output = write_slice_and_move_forward(b"\",\"elts\":[", output);
    let mut i = 0;
    while i < elts.len() {
        if i > 0 {
            output = write_slice_and_move_forward(b",", output);
        }
        output = write_expr_and_move_forward(&elts[i], output);
        i += 1;
    }
    write_slice_and_move_forward(b"]}", output)
}

const fn serialized_container_len_for_introspection(elts: &[PyStaticExpr]) -> usize {
    let mut len = 21;
    let mut i = 0;
    while i < elts.len() {
        if i > 0 {
            len += 1;
        }
        len += serialized_len_for_introspection(&elts[i]);
        i += 1;
    }
    len
}

const fn serialized_json_string_len(value: &str) -> usize {
    let value = value.as_bytes();
    let mut len = 2;
    let mut i = 0;
    while i < value.len() {
        len += match value[i] {
            b'\\' | b'"' | 0x08 | 0x0C | b'\n' | b'\r' | b'\t' => 2,
            0..32 => 6,
            _ => 1,
        };
        i += 1;
    }
    len
}

fn fmt_elements(elts: &[PyStaticExpr], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, elt) in elts.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        elt.fmt(f)?;
    }
    Ok(())
}

/// The full name of a `#[pyclass]` inside a [`PyStaticExpr`].
///
/// To get the underlying [`PyStaticExpr`] use [`expr`](PyClassNameStaticExpr::expr).
#[derive(Clone, Copy)]
pub struct PyClassNameStaticExpr {
    expr: &'static PyStaticExpr,
    introspection_id: &'static str,
}

impl PyClassNameStaticExpr {
    #[doc(hidden)]
    #[inline]
    pub const fn new(expr: &'static PyStaticExpr, introspection_id: &'static str) -> Self {
        Self {
            expr,
            introspection_id,
        }
    }

    /// The pyclass type as an expression like `module.name`
    ///
    /// This is based on the `name` and `module` parameter of the `#[pyclass]` macro.
    /// The `module` part might not be a valid module from which the type can be imported.
    #[inline]
    pub const fn expr(&self) -> &'static PyStaticExpr {
        self.expr
    }
}

impl AsRef<PyStaticExpr> for PyClassNameStaticExpr {
    #[inline]
    fn as_ref(&self) -> &PyStaticExpr {
        self.expr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        const T: PyStaticExpr = type_hint_subscript!(
            type_hint_identifier!("builtins", "dict"),
            type_hint_union!(
                type_hint_identifier!("builtins", "int"),
                type_hint_subscript!(
                    type_hint_identifier!("typing", "Literal"),
                    PyStaticExpr::Constant {
                        value: PyStaticConstant::Str("\0\t\\\"")
                    }
                )
            ),
            type_hint_identifier!("datetime", "time")
        );
        assert_eq!(
            T.to_string(),
            "dict[int | typing.Literal[\"\\0\\t\\\\\\\"\"], datetime.time]"
        )
    }

    #[test]
    fn test_serialize_for_introspection() {
        fn check_serialization(expr: PyStaticExpr, expected: &str) {
            let mut out = vec![0; serialized_len_for_introspection(&expr)];
            serialize_for_introspection(&expr, &mut out);
            assert_eq!(std::str::from_utf8(&out).unwrap(), expected)
        }

        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::None,
            },
            r#"{"type":"constant","kind":"none"}"#,
        );
        check_serialization(
            type_hint_identifier!("builtins", "int"),
            r#"{"type":"name","id":"int"}"#,
        );
        check_serialization(
            type_hint_identifier!("datetime", "date"),
            r#"{"type":"attribute","value":{"type":"name","id":"datetime"},"attr":"date"}"#,
        );
        check_serialization(
            type_hint_union!(
                type_hint_identifier!("builtins", "int"),
                type_hint_identifier!("builtins", "float")
            ),
            r#"{"type":"binop","left":{"type":"name","id":"int"},"op":"bitor","right":{"type":"name","id":"float"}}"#,
        );
        check_serialization(
            PyStaticExpr::Tuple {
                elts: &[type_hint_identifier!("builtins", "list")],
            },
            r#"{"type":"tuple","elts":[{"type":"name","id":"list"}]}"#,
        );
        check_serialization(
            PyStaticExpr::List {
                elts: &[type_hint_identifier!("builtins", "list")],
            },
            r#"{"type":"list","elts":[{"type":"name","id":"list"}]}"#,
        );
        check_serialization(
            type_hint_subscript!(
                type_hint_identifier!("builtins", "list"),
                type_hint_identifier!("builtins", "int")
            ),
            r#"{"type":"subscript","value":{"type":"name","id":"list"},"slice":{"type":"name","id":"int"}}"#,
        );
        check_serialization(
            PyStaticExpr::PyClass(PyClassNameStaticExpr::new(
                &type_hint_identifier!("builtins", "foo"),
                "foo",
            )),
            r#"{"type":"id","id":"foo"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Bool(true),
            },
            r#"{"type":"constant","kind":"bool","value":true}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Bool(false),
            },
            r#"{"type":"constant","kind":"bool","value":false}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Int("-123"),
            },
            r#"{"type":"constant","kind":"int","value":"-123"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Float("-2.1"),
            },
            r#"{"type":"constant","kind":"float","value":"-2.1"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Float("+2.1e10"),
            },
            r#"{"type":"constant","kind":"float","value":"+2.1e10"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Str("abc(1)"),
            },
            r#"{"type":"constant","kind":"str","value":"abc(1)"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Str("\"\\/\x08\x0C\n\r\t\0\x19a"),
            },
            r#"{"type":"constant","kind":"str","value":"\"\\/\b\f\n\r\t\u0000\u0019a"}"#,
        );
        check_serialization(
            PyStaticExpr::Constant {
                value: PyStaticConstant::Ellipsis,
            },
            r#"{"type":"constant","kind":"ellipsis"}"#,
        );
    }
}
