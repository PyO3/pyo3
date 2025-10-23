//! Runtime inspection of objects exposed to Python.
//!
//! Tracking issue: <https://github.com/PyO3/pyo3/issues/2454>.

use std::fmt;
use std::fmt::Formatter;

pub mod types;

/// A [type hint](https://docs.python.org/3/glossary.html#term-type-hint).
///
/// This struct aims at being used in `const` contexts like in [`FromPyObject::INPUT_TYPE`](crate::FromPyObject::INPUT_TYPE) and [`IntoPyObject::OUTPUT_TYPE`](crate::IntoPyObject::OUTPUT_TYPE).
///
/// ```
/// use pyo3::inspect::TypeHint;
///
/// const T: TypeHint = TypeHint::union(&[TypeHint::builtin("int"), TypeHint::module_attr("b", "B")]);
/// assert_eq!(T.to_string(), "int | b.B");
/// ```
#[derive(Clone, Copy)]
pub struct TypeHint {
    inner: TypeHintExpr,
}

#[derive(Clone, Copy)]
enum TypeHintExpr {
    /// A built-name like `list` or `datetime`. Used for built-in types or modules.
    Builtin { id: &'static str },
    /// A module member like `datetime.time` where module = `datetime` and attr = `time`
    ModuleAttribute {
        module: &'static str,
        attr: &'static str,
    },
    /// A union elts[0] | ... | elts[len]
    Union { elts: &'static [TypeHint] },
    /// A subscript main[*args]
    Subscript {
        value: &'static TypeHint,
        slice: &'static [TypeHint],
    },
}

impl TypeHint {
    /// A builtin like `int` or `list`
    ///
    /// ```
    /// use pyo3::inspect::TypeHint;
    ///
    /// const T: TypeHint = TypeHint::builtin("int");
    /// assert_eq!(T.to_string(), "int");
    /// ```
    pub const fn builtin(name: &'static str) -> Self {
        Self {
            inner: TypeHintExpr::Builtin { id: name },
        }
    }

    /// A type contained in a module like `datetime.time`
    ///
    /// ```
    /// use pyo3::inspect::TypeHint;
    ///
    /// const T: TypeHint = TypeHint::module_attr("datetime", "time");
    /// assert_eq!(T.to_string(), "datetime.time");
    /// ```
    pub const fn module_attr(module: &'static str, attr: &'static str) -> Self {
        Self {
            inner: TypeHintExpr::ModuleAttribute { module, attr },
        }
    }

    /// The union of multiple types
    ///
    /// ```
    /// use pyo3::inspect::TypeHint;
    ///
    /// const T: TypeHint = TypeHint::union(&[TypeHint::builtin("int"), TypeHint::builtin("float")]);
    /// assert_eq!(T.to_string(), "int | float");
    /// ```
    pub const fn union(elts: &'static [TypeHint]) -> Self {
        Self {
            inner: TypeHintExpr::Union { elts },
        }
    }

    /// A subscribed type, often a container
    ///
    /// ```
    /// use pyo3::inspect::TypeHint;
    ///
    /// const T: TypeHint = TypeHint::subscript(&TypeHint::builtin("dict"), &[TypeHint::builtin("int"), TypeHint::builtin("str")]);
    /// assert_eq!(T.to_string(), "dict[int, str]");
    /// ```
    pub const fn subscript(value: &'static Self, slice: &'static [Self]) -> Self {
        Self {
            inner: TypeHintExpr::Subscript { value, slice },
        }
    }
}

/// Serialize the type for introspection and return the number of written bytes
///
/// We use the same AST as Python: https://docs.python.org/3/library/ast.html#abstract-grammar
#[doc(hidden)]
pub const fn serialize_for_introspection(hint: &TypeHint, mut output: &mut [u8]) -> usize {
    let original_len = output.len();
    match &hint.inner {
        TypeHintExpr::Builtin { id } => {
            output = write_slice_and_move_forward(b"{\"type\":\"builtin\",\"id\":\"", output);
            output = write_slice_and_move_forward(id.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        TypeHintExpr::ModuleAttribute { module, attr } => {
            output = write_slice_and_move_forward(b"{\"type\":\"attribute\",\"module\":\"", output);
            output = write_slice_and_move_forward(module.as_bytes(), output);
            output = write_slice_and_move_forward(b"\",\"attr\":\"", output);
            output = write_slice_and_move_forward(attr.as_bytes(), output);
            output = write_slice_and_move_forward(b"\"}", output);
        }
        TypeHintExpr::Union { elts } => {
            output = write_slice_and_move_forward(b"{\"type\":\"union\",\"elts\":[", output);
            let mut i = 0;
            while i < elts.len() {
                if i > 0 {
                    output = write_slice_and_move_forward(b",", output);
                }
                output = write_type_hint_and_move_forward(&elts[i], output);
                i += 1;
            }
            output = write_slice_and_move_forward(b"]}", output);
        }
        TypeHintExpr::Subscript { value, slice } => {
            output = write_slice_and_move_forward(b"{\"type\":\"subscript\",\"value\":", output);
            output = write_type_hint_and_move_forward(value, output);
            output = write_slice_and_move_forward(b",\"slice\":[", output);
            let mut i = 0;
            while i < slice.len() {
                if i > 0 {
                    output = write_slice_and_move_forward(b",", output);
                }
                output = write_type_hint_and_move_forward(&slice[i], output);
                i += 1;
            }
            output = write_slice_and_move_forward(b"]}", output);
        }
    }
    original_len - output.len()
}

/// Length required by [`Self::serialize_for_introspection`]
#[doc(hidden)]
pub const fn serialized_len_for_introspection(hint: &TypeHint) -> usize {
    match &hint.inner {
        TypeHintExpr::Builtin { id } => 26 + id.len(),
        TypeHintExpr::ModuleAttribute { module, attr } => 42 + module.len() + attr.len(),
        TypeHintExpr::Union { elts } => {
            let mut count = 26;
            let mut i = 0;
            while i < elts.len() {
                if i > 0 {
                    count += 1;
                }
                count += serialized_len_for_introspection(&elts[i]);
                i += 1;
            }
            count
        }
        TypeHintExpr::Subscript { value, slice } => {
            let mut count = 40 + serialized_len_for_introspection(value);
            let mut i = 0;
            while i < slice.len() {
                if i > 0 {
                    count += 1;
                }
                count += serialized_len_for_introspection(&slice[i]);
                i += 1;
            }
            count
        }
    }
}

impl fmt::Display for TypeHint {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.inner {
            TypeHintExpr::Builtin { id } => id.fmt(f),
            TypeHintExpr::ModuleAttribute { module, attr } => {
                module.fmt(f)?;
                f.write_str(".")?;
                attr.fmt(f)
            }
            TypeHintExpr::Union { elts } => {
                for (i, elt) in elts.iter().enumerate() {
                    if i > 0 {
                        f.write_str(" | ")?;
                    }
                    elt.fmt(f)?;
                }
                Ok(())
            }
            TypeHintExpr::Subscript { value, slice } => {
                value.fmt(f)?;
                f.write_str("[")?;
                for (i, elt) in slice.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    elt.fmt(f)?;
                }
                f.write_str("]")
            }
        }
    }
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

const fn write_type_hint_and_move_forward<'a>(
    value: &TypeHint,
    output: &'a mut [u8],
) -> &'a mut [u8] {
    let written = serialize_for_introspection(value, output);
    output.split_at_mut(written).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        const T: TypeHint = TypeHint::subscript(
            &TypeHint::builtin("dict"),
            &[
                TypeHint::union(&[TypeHint::builtin("int"), TypeHint::builtin("float")]),
                TypeHint::module_attr("datetime", "time"),
            ],
        );
        assert_eq!(T.to_string(), "dict[int | float, datetime.time]")
    }

    #[test]
    fn test_serialize_for_introspection() {
        const T: TypeHint = TypeHint::subscript(
            &TypeHint::builtin("dict"),
            &[
                TypeHint::union(&[TypeHint::builtin("int"), TypeHint::builtin("float")]),
                TypeHint::module_attr("datetime", "time"),
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
            r#"{"type":"subscript","value":{"type":"builtin","id":"dict"},"slice":[{"type":"union","elts":[{"type":"builtin","id":"int"},{"type":"builtin","id":"float"}]},{"type":"attribute","module":"datetime","attr":"time"}]}"#
        )
    }
}
