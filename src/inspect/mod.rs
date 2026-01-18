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
    // TODO: add Bool(bool), String(&'static str)... (is useful for Literal["foo", "bar"] types)
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
                type_hint_identifier!("builtins", "float")
            ),
            type_hint_identifier!("datetime", "time")
        );
        assert_eq!(T.to_string(), "dict[int | float, datetime.time]")
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
        )
    }
}
