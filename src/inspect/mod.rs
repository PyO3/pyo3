//! Runtime inspection of objects exposed to Python.
//!
//! Tracking issue: <https://github.com/PyO3/pyo3/issues/2454>.

use std::fmt;
use std::fmt::Formatter;

pub mod types;

/// A [type hint](https://docs.python.org/3/glossary.html#term-type-hint) with a list of imports to make it valid
///
/// This struct aims at being used in `const` contexts like in [`FromPyObject::INPUT_TYPE`](crate::FromPyObject::INPUT_TYPE) and [`IntoPyObject::OUTPUT_TYPE`](crate::IntoPyObject::OUTPUT_TYPE).
///
/// ```
/// use pyo3::{type_hint, type_hint_union};
/// use pyo3::inspect::TypeHint;
///
/// const T: TypeHint = type_hint_union!(type_hint!("int"), type_hint!("b", "B"));
/// assert_eq!(T.to_string(), "int | B");
/// ```
#[derive(Clone, Copy)]
pub struct TypeHint {
    /// The type hint annotation
    #[doc(hidden)]
    pub annotation: &'static str,
    /// The modules to import
    #[doc(hidden)]
    pub imports: &'static [TypeHintImport],
}

/// `from {module} import {name}` import
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct TypeHintImport {
    /// The module from which to import
    #[doc(hidden)]
    pub module: &'static str,
    /// The elements to import from the module
    #[doc(hidden)]
    pub name: &'static str,
}

impl fmt::Display for TypeHint {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.annotation.fmt(f)
    }
}

/// Allows to build a [`TypeHint`] from a module name and a qualified name
///
/// ```
/// use pyo3::type_hint;
/// use pyo3::inspect::TypeHint;
///
/// const T: TypeHint = type_hint!("collections.abc", "Sequence");
/// assert_eq!(T.to_string(), "Sequence");
/// ```
#[macro_export]
macro_rules! type_hint {
    ($qualname: expr) => {
        $crate::inspect::TypeHint {
            annotation: $qualname,
            imports: &[],
        }
    };
    ($module:expr, $name: expr) => {
        $crate::inspect::TypeHint {
            annotation: $name,
            imports: &[$crate::inspect::TypeHintImport {
                module: $module,
                name: $name,
            }],
        }
    };
}

/// Allows to build a [`TypeHint`] that is the union of other [`TypeHint`]
///
/// ```
/// use pyo3::{type_hint, type_hint_union};
/// use pyo3::inspect::TypeHint;
///
/// const T: TypeHint = type_hint_union!(type_hint!("a", "A"), type_hint!("b", "B"));
/// assert_eq!(T.to_string(), "A | B");
/// ```
#[macro_export]
macro_rules! type_hint_union {
    // TODO: avoid using the parameters twice
    // TODO: factor our common code in const functions
    ($arg:expr) => { $arg };
    ($firstarg:expr, $($arg:expr),+) => {{
        $crate::inspect::TypeHint {
            annotation: {
                const PARTS: &[&[u8]] = &[$firstarg.annotation.as_bytes(), $(b" | ", $arg.annotation.as_bytes()),*];
                unsafe {
                    ::std::str::from_utf8_unchecked(&$crate::impl_::concat::combine_to_array::<{
                        $crate::impl_::concat::combined_len(PARTS)
                    }>(PARTS))
                }
            },
            imports: {
                const ARGS: &[$crate::inspect::TypeHint] = &[$firstarg, $($arg),*];
                const LEN: usize = {
                    let mut count = 0;
                    let mut i = 0;
                    while i < ARGS.len() {
                        count += ARGS[i].imports.len();
                        i += 1;
                    }
                    count
                };
                const OUTPUT: [$crate::inspect::TypeHintImport; LEN] = {
                    let mut output = [$crate::inspect::TypeHintImport { module: "", name: "" }; LEN];
                    let mut args_i = 0;
                    let mut in_arg_i = 0;
                    let mut output_i = 0;
                    while args_i < ARGS.len() {
                        while in_arg_i < ARGS[args_i].imports.len() {
                            output[output_i] = ARGS[args_i].imports[in_arg_i];
                            in_arg_i += 1;
                            output_i += 1;
                        }
                        args_i += 1;
                        in_arg_i = 0;
                    }
                    output
                };
                &OUTPUT
            }
        }
    }};
}

/// Allows to build a [`TypeHint`] that is the subscripted
///
/// ```
/// use pyo3::{type_hint, type_hint_subscript};
/// use pyo3::inspect::TypeHint;
///
/// const T: TypeHint = type_hint_subscript!(type_hint!("collections.abc", "Sequence"), type_hint!("weakref", "ProxyType"));
/// assert_eq!(T.to_string(), "Sequence[ProxyType]");
/// ```
#[macro_export]
macro_rules! type_hint_subscript {
    // TODO: avoid using the parameters twice
    // TODO: factor our common code in const functions
    ($main:expr, $firstarg:expr $(, $arg:expr)*) => {{
        $crate::inspect::TypeHint {
            annotation: {
                const PARTS: &[&[u8]] = &[$main.annotation.as_bytes(), b"[", $firstarg.annotation.as_bytes() $(, b", ", $arg.annotation.as_bytes())* , b"]"];
                unsafe {
                    ::std::str::from_utf8_unchecked(&$crate::impl_::concat::combine_to_array::<{
                        $crate::impl_::concat::combined_len(PARTS)
                    }>(PARTS))
                }
            },
            imports: {
                const ARGS: &[$crate::inspect::TypeHint] = &[$main, $firstarg, $($arg),*];
                const LEN: usize = {
                    let mut count = 0;
                    let mut i = 0;
                    while i < ARGS.len() {
                        count += ARGS[i].imports.len();
                        i += 1;
                    }
                    count
                };
                const OUTPUT: [$crate::inspect::TypeHintImport; LEN] = {
                    let mut output = [$crate::inspect::TypeHintImport { module: "", name: "" }; LEN];
                    let mut args_i = 0;
                    let mut in_arg_i = 0;
                    let mut output_i = 0;
                    while args_i < ARGS.len() {
                        while in_arg_i < ARGS[args_i].imports.len() {
                            output[output_i] = ARGS[args_i].imports[in_arg_i];
                            in_arg_i += 1;
                            output_i += 1;
                        }
                        args_i += 1;
                        in_arg_i = 0;
                    }
                    output
                };
                &OUTPUT
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union() {
        const T: TypeHint = type_hint_union!(type_hint!("a", "A"), type_hint!("b", "B"));
        assert_eq!(T.annotation, "A | B");
        assert_eq!(T.imports[0].name, "A");
        assert_eq!(T.imports[0].module, "a");
        assert_eq!(T.imports[1].name, "B");
        assert_eq!(T.imports[1].module, "b");
    }

    #[test]
    fn test_subscript() {
        const T: TypeHint = type_hint_subscript!(
            type_hint!("collections.abc", "Sequence"),
            type_hint!("weakref", "ProxyType")
        );
        assert_eq!(T.annotation, "Sequence[ProxyType]");
        assert_eq!(T.imports[0].name, "Sequence");
        assert_eq!(T.imports[0].module, "collections.abc");
        assert_eq!(T.imports[1].name, "ProxyType");
        assert_eq!(T.imports[1].module, "weakref");
    }
}
