//! Runtime inspection of objects exposed to Python.
//!
//! Tracking issue: <https://github.com/PyO3/pyo3/issues/2454>.

use std::fmt::{self, Display, Write};

pub mod types;

/// A Python expression.
///
/// This is the `expr` production of the [Python `ast` module grammar](https://docs.python.org/3/library/ast.html#abstract-grammar)
///
/// This struct aims at being used in `const` contexts like in [`FromPyObject::INPUT_TYPE`](crate::FromPyObject::INPUT_TYPE) and [`IntoPyObject::OUTPUT_TYPE`](crate::IntoPyObject::OUTPUT_TYPE).
///
/// ```
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = PyStaticExpr::union(&[PyStaticExpr::builtin("int"), PyStaticExpr::attribute(&PyStaticExpr::module("b"), "B")]);
/// assert_eq!(T.to_string(), "int | b.B");
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum PyStaticExpr {
    /// A constant like `None` or `123`
    Constant { value: PyStaticConstant },
    /// A name
    Name {
        id: &'static str,
        kind: PyStaticNameKind,
    },
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
    /// A subscript `main[*args]`
    Subscript {
        value: &'static Self,
        slice: &'static Self,
    },
}

impl PyStaticExpr {
    /// A value in the local context. Can be a locally defined class, a module relative to the current one...
    #[doc(hidden)]
    pub const fn local(id: &'static str) -> Self {
        Self::Name {
            id,
            kind: PyStaticNameKind::Local,
        }
    }

    /// A global builtin like `int` or `list`
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::builtin("int");
    /// assert_eq!(T.to_string(), "int");
    /// ```
    pub const fn builtin(id: &'static str) -> Self {
        Self::Name {
            id,
            kind: PyStaticNameKind::Global,
        }
    }

    /// An absolute module name like `datetime` or `collections.abc`
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::module("datetime");
    /// assert_eq!(T.to_string(), "datetime");
    /// ```
    pub const fn module(id: &'static str) -> Self {
        Self::Name {
            id,
            kind: PyStaticNameKind::Global,
        }
    }

    /// A type contained in a module like `datetime.time`
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::attribute(&PyStaticExpr::module("datetime"), "time");
    /// assert_eq!(T.to_string(), "datetime.time");
    /// ```
    pub const fn attribute(value: &'static Self, attr: &'static str) -> Self {
        Self::Attribute { value, attr }
    }

    /// The bit or (`|`) operator, also used for type union
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::bit_or(&PyStaticExpr::builtin("int"), &PyStaticExpr::builtin("float"));
    /// assert_eq!(T.to_string(), "int | float");
    /// ```
    pub const fn bit_or(left: &'static Self, right: &'static Self) -> Self {
        Self::BinOp {
            left,
            op: PyStaticOperator::BitOr,
            right,
        }
    }

    /// A tuple
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::tuple(&[PyStaticExpr::builtin("int"), PyStaticExpr::builtin("str")]);
    /// assert_eq!(T.to_string(), "(int, str)");
    /// ```
    pub const fn tuple(elts: &'static [Self]) -> Self {
        Self::Tuple { elts }
    }

    /// A list
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::subscript(
    ///     &PyStaticExpr::builtin("Callable"),
    ///     &PyStaticExpr::tuple(&[
    ///         &PyStaticExpr::list(&[PyStaticExpr::builtin("int")]),
    ///         PyStaticExpr::builtin("str")
    ///     ])
    /// );
    /// assert_eq!(T.to_string(), "Callable[[int], str]");
    /// ```
    pub const fn list(elts: &'static [Self]) -> Self {
        Self::List { elts }
    }

    /// A subscribed expression
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::subscript(&PyStaticExpr::builtin("list"), &PyStaticExpr::builtin("int")));
    /// assert_eq!(T.to_string(), "list[int");
    /// ```
    pub const fn subscript(value: &'static Self, slice: &'static Self) -> Self {
        Self::Subscript { value, slice }
    }

    /// The `None` constant
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::none();
    /// assert_eq!(T.to_string(), "None");
    /// ```
    #[doc(hidden)]
    pub const fn none() -> Self {
        Self::Constant {
            value: PyStaticConstant::None,
        }
    }
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
        PyStaticExpr::Name { id, kind } => {
            output = write_slice_and_move_forward(b"{\"type\":\"", output);
            output = write_slice_and_move_forward(
                match kind {
                    PyStaticNameKind::Local => b"local",
                    PyStaticNameKind::Global => b"global",
                },
                output,
            );
            output = write_slice_and_move_forward(b"\",\"id\":\"", output);
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
    }
    original_len - output.len()
}

/// Length required by [`serialize_for_introspection`]
#[doc(hidden)]
pub const fn serialized_len_for_introspection(expr: &PyStaticExpr) -> usize {
    match expr {
        PyStaticExpr::Constant { value } => match value {
            PyStaticConstant::None => 34,
        },
        PyStaticExpr::Name { id, kind } => {
            (match kind {
                PyStaticNameKind::Local => 24,
                PyStaticNameKind::Global => 25,
            }) + id.len()
        }
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
            39 + serialized_len_for_introspection(value) + serialized_len_for_introspection(slice)
        }
    }
}

impl fmt::Display for PyStaticExpr {
    #[inline]
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
                f.write_str("[")?;
                if let PyStaticExpr::Tuple { elts } = slice {
                    // We don't display the tuple parentheses
                    fmt_elements(elts, f)?;
                } else {
                    slice.fmt(f)?;
                }
                f.write_str("]")
            }
        }
    }
}

/// A PyO3 extension to the Python AST to know more about [`PyStaticExpr::Name`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum PyStaticNameKind {
    /// A local name, relative to the current module
    Local,
    /// A global name, can be a module like `datetime`, a builtin like `int`...
    Global,
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

/// An operator used in [`PyStaticExpr::BinaryOpt`].
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
    let mut len = 20;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        const T: PyStaticExpr = PyStaticExpr::subscript(
            &PyStaticExpr::builtin("dict"),
            &PyStaticExpr::tuple(&[
                PyStaticExpr::bit_or(&PyStaticExpr::builtin("int"), &PyStaticExpr::local("weird")),
                PyStaticExpr::attribute(&PyStaticExpr::module("datetime"), "time"),
            ]),
        );
        assert_eq!(T.to_string(), "dict[int | weird, datetime.time]")
    }

    #[test]
    fn test_serialize_for_introspection() {
        const T: PyStaticExpr = PyStaticExpr::subscript(
            &PyStaticExpr::attribute(&PyStaticExpr::module("typing"), "Callable"),
            &PyStaticExpr::tuple(&[
                PyStaticExpr::list(&[
                    PyStaticExpr::bit_or(
                        &PyStaticExpr::builtin("int"),
                        &PyStaticExpr::local("weird"),
                    ),
                    PyStaticExpr::attribute(&PyStaticExpr::module("datetime"), "time"),
                ]),
                PyStaticExpr::none(),
            ]),
        );
        const SER_LEN: usize = serialized_len_for_introspection(&T);
        const SER: [u8; SER_LEN] = {
            let mut out: [u8; SER_LEN] = [0; SER_LEN];
            serialize_for_introspection(&T, &mut out);
            out
        };
        assert_eq!(
            std::str::from_utf8(&SER).unwrap(),
            r#"{"type":"subscript","value":{"type":"attribute","value":{"type":"global","id":"typing"},"attr":"Callable"},"slice":{"type":"tuple","elts":[{"type":"list","elts":[{"type":"binop","left":{"type":"global","id":"int"},"op":"bitor","right":{"type":"local","id":"weird"}},{"type":"attribute","value":{"type":"global","id":"datetime"},"attr":"time"}]},{"type":"constant","kind":"none"}]}}"#
        )
    }
}
