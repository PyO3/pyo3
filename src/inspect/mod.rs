//! Runtime inspection of objects exposed to Python.
//!
//! Tracking issue: <https://github.com/PyO3/pyo3/issues/2454>.

use std::fmt;
use std::fmt::Write;

pub mod types;

/// A Python expression. TODO: link to as
///
/// This struct aims at being used in `const` contexts like in [`FromPyObject::INPUT_TYPE`](crate::FromPyObject::INPUT_TYPE) and [`IntoPyObject::OUTPUT_TYPE`](crate::IntoPyObject::OUTPUT_TYPE).
///
/// ```
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = PyStaticExpr::union(&[PyStaticExpr::builtin("int"), PyStaticExpr::module_attr("b", "B")]);
/// assert_eq!(T.to_string(), "int | b.B");
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum PyStaticExpr {
    /// A local name. Used only when the module is unknown.
    Local { id: &'static str },
    /// A built-name like `list` or `datetime`. Used for built-in types or modules.
    Builtin { id: &'static str },
    /// A module member like `datetime.time` where module = `datetime` and attr = `time`
    ModuleAttribute {
        module: &'static str,
        attr: &'static str,
    },
    /// A binary operator
    BinOp {
        left: &'static PyStaticExpr,
        op: Operator,
        right: &'static PyStaticExpr,
    },
    /// A tuple
    Tuple { elts: &'static [Self] },
    /// A subscript `main[*args]`
    Subscript {
        value: &'static PyStaticExpr,
        slice: &'static PyStaticExpr,
    },
}

impl PyStaticExpr {
    /// A builtin like `int` or `list`
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::builtin("int");
    /// assert_eq!(T.to_string(), "int");
    /// ```
    pub const fn builtin(name: &'static str) -> Self {
        Self::Builtin { id: name }
    }

    /// A type contained in a module like `datetime.time`
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::module_attr("datetime", "time");
    /// assert_eq!(T.to_string(), "datetime.time");
    /// ```
    pub const fn module_attr(module: &'static str, attr: &'static str) -> Self {
        if matches!(module.as_bytes(), b"builtins") {
            Self::Builtin { id: attr }
        } else {
            Self::ModuleAttribute { module, attr }
        }
    }

    /// A value in the local module which module is unknown
    #[doc(hidden)]
    pub const fn local(name: &'static str) -> Self {
        Self::Local { id: name }
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
            op: Operator::BitOr,
            right,
        }
    }

    /// A tuple
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::subscript(&PyStaticExpr::builtin("dict"), &[PyStaticExpr::builtin("int"), PyStaticExpr::builtin("str")]);
    /// assert_eq!(T.to_string(), "dict[int, str]");
    /// ```
    pub const fn tuple(elts: &'static [Self]) -> Self {
        Self::Tuple { elts }
    }

    /// A subscribed expression
    ///
    /// ```
    /// use pyo3::inspect::PyStaticExpr;
    ///
    /// const T: PyStaticExpr = PyStaticExpr::subscript(&PyStaticExpr::builtin("dict"), &PyStaticExpr::slice(&[PyStaticExpr::builtin("int"), PyStaticExpr::builtin("str")]));
    /// assert_eq!(T.to_string(), "dict[int, str]");
    /// ```
    pub const fn subscript(value: &'static Self, slice: &'static Self) -> Self {
        Self::Subscript { value, slice }
    }
}

/// Serialize the type for introspection and return the number of written bytes
///
/// We use the same AST as Python: <https://docs.python.org/3/library/ast.html#abstract-grammar>
#[doc(hidden)]
pub const fn serialize_for_introspection(expr: &PyStaticExpr, output: &mut [u8]) -> usize {
    let original_len = output.len();
    match expr {
        PyStaticExpr::Local { id } => {
            output = write_slice_and_move_forward(b"{\"type\":\"local\",\"id\":\"", output);
            output = write_slice_and_move_forward(id.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        PyStaticExpr::Builtin { id } => {
            output = write_slice_and_move_forward(b"{\"type\":\"builtin\",\"id\":\"", output);
            output = write_slice_and_move_forward(id.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        PyStaticExpr::ModuleAttribute { module, attr } => {
            output = write_slice_and_move_forward(b"{\"type\":\"attribute\",\"module\":\"", output);
            output = write_slice_and_move_forward(module.as_bytes(), output);
            output = write_slice_and_move_forward(b"\",\"attr\":\"", output);
            output = write_slice_and_move_forward(attr.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        PyStaticExpr::BinOp { left, right, .. } => {
            output = write_slice_and_move_forward(b"{\"type\":\"union\",\"elts\":[", output);
            output = write_expr_and_move_forward(left, output);
            output = write_slice_and_move_forward(b",", output);
            output = write_expr_and_move_forward(right, output);
            output = write_slice_and_move_forward(b"]}", output);
        }
        PyStaticExpr::Tuple { elts } => {
            output = write_slice_and_move_forward(b"{\"type\":\"tuple\",\"elts\":[", output);
            let mut i = 0;
            while i < elts.len() {
                if i > 0 {
                    output = write_slice_and_move_forward(b",", output);
                }
                output = write_expr_and_move_forward(&elts[i], output);
                i += 1;
            }
            output = write_slice_and_move_forward(b"]}", output);
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
        PyStaticExpr::Local { id } => 24 + id.len(),
        PyStaticExpr::Builtin { id } => 26 + id.len(),
        PyStaticExpr::ModuleAttribute { module, attr } => 42 + module.len() + attr.len(),
        PyStaticExpr::BinOp { left, right, .. } => {
            27 + serialized_len_for_introspection(left) + serialized_len_for_introspection(right)
        }
        PyStaticExpr::Tuple { elts } => 0,
        PyStaticExpr::Subscript { value, slice } => 0,
    }
}

impl fmt::Display for PyStaticExpr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin { id } | Self::Local { id } => id.fmt(f),
            Self::ModuleAttribute { module, attr } => {
                module.fmt(f)?;
                f.write_str(".")?;
                attr.fmt(f)
            }
            Self::BinOp { left, op, right } => {
                left.fmt(f)?;
                f.write_char(' ')?;
                f.write_char(match op {
                    Operator::BitOr => '|',
                })?;
                f.write_char(' ')?;
                right.fmt(f)
            }
            Self::Tuple { elts } => {
                f.write_char('[')?;
                fmt_elements(elts, f)?;
                f.write_char(']')
            }
            Self::Subscript { value, slice } => {
                value.fmt(f)?;
                f.write_str("[")?;
                if let PyStaticExpr::Tuple { elts } = slice {
                    // We don't display the tuple parentheses
                    TODO
                } else {
                    slice.fmt(f)?;
                }
                f.write_str("]")
            }
        }
    }
}

/// An operator used in [`PyStaticExpr::BinaryOpt`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Operator {
    BitOr, // TODO: naming
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
    let written = serialize_expr_for_introspection(value, output);
    output.split_at_mut(written).1
}

fn fmt_elements(elts: &[PyStaticExpr], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, elt) in elts.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
            elt.fmt(f)?;
        }
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
            &[
                PyStaticExpr::union(&[
                    PyStaticExpr::builtin("int"),
                    PyStaticExpr::module_attr("builtins", "float"),
                    PyStaticExpr::local("weird"),
                ]),
                PyStaticExpr::module_attr("datetime", "time"),
            ],
        );
        assert_eq!(T.to_string(), "dict[int | float | weird, datetime.time]")
    }

    #[test]
    fn test_serialize_for_introspection() {
        const T: PyStaticExpr = PyStaticExpr::subscript(
            &PyStaticExpr::builtin("dict"),
            &[
                PyStaticExpr::bit_or(&PyStaticExpr::builtin("int"), &PyStaticExpr::local("weird")),
                PyStaticExpr::module_attr("datetime", "time"),
            ],
        );
        const SER_LEN: usize = serialized_len_for_introspection(&T);
        const SER: [u8; SER_LEN] = {
            let mut out: [u8; SER_LEN] = [0; SER_LEN];
            serialize_for_introspection(&T, &mut out);
            out
        };
        assert_eq!(
            std::str::from_utf8(&SER).unwrap(),
            r#"{"type":"subscript","value":{"type":"builtin","id":"dict"},"slice":[{"type":"union","elts":[{"type":"builtin","id":"int"},{"type":"local","id":"weird"}]},{"type":"attribute","module":"datetime","attr":"time"}]}"#
        )
    }
}
