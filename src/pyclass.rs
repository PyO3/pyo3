//! `PyClass` and related traits.
use crate::{ffi, impl_::pyclass::PyClassImpl, PyTypeInfo};
use std::{cmp::Ordering, os::raw::c_int};

mod create_type_object;
mod gc;
mod guard;

pub(crate) use self::create_type_object::{create_type_object, PyClassTypeObject};

pub use self::gc::{PyTraverseError, PyVisit};
pub use self::guard::{PyClassGuard, PyClassGuardMap, PyClassGuardMut};

/// Types that can be used as Python classes.
///
/// The `#[pyclass]` attribute implements this trait for your Rust struct -
/// you shouldn't implement this trait directly.
pub trait PyClass: PyTypeInfo + PyClassImpl {
    /// Whether the pyclass is frozen.
    ///
    /// This can be enabled via `#[pyclass(frozen)]`.
    type Frozen: Frozen;
}

/// Operators for the `__richcmp__` method
#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    /// The *less than* operator.
    Lt = ffi::Py_LT as isize,
    /// The *less than or equal to* operator.
    Le = ffi::Py_LE as isize,
    /// The equality operator.
    Eq = ffi::Py_EQ as isize,
    /// The *not equal to* operator.
    Ne = ffi::Py_NE as isize,
    /// The *greater than* operator.
    Gt = ffi::Py_GT as isize,
    /// The *greater than or equal to* operator.
    Ge = ffi::Py_GE as isize,
}

impl CompareOp {
    /// Conversion from the C enum.
    pub fn from_raw(op: c_int) -> Option<Self> {
        match op {
            ffi::Py_LT => Some(CompareOp::Lt),
            ffi::Py_LE => Some(CompareOp::Le),
            ffi::Py_EQ => Some(CompareOp::Eq),
            ffi::Py_NE => Some(CompareOp::Ne),
            ffi::Py_GT => Some(CompareOp::Gt),
            ffi::Py_GE => Some(CompareOp::Ge),
            _ => None,
        }
    }

    /// Returns if a Rust [`std::cmp::Ordering`] matches this ordering query.
    ///
    /// Usage example:
    ///
    /// ```rust,no_run
    /// # use pyo3::prelude::*;
    /// # use pyo3::class::basic::CompareOp;
    ///
    /// #[pyclass]
    /// struct Size {
    ///     size: usize,
    /// }
    ///
    /// #[pymethods]
    /// impl Size {
    ///     fn __richcmp__(&self, other: &Size, op: CompareOp) -> bool {
    ///         op.matches(self.size.cmp(&other.size))
    ///     }
    /// }
    /// ```
    pub fn matches(&self, result: Ordering) -> bool {
        match self {
            CompareOp::Eq => result == Ordering::Equal,
            CompareOp::Ne => result != Ordering::Equal,
            CompareOp::Lt => result == Ordering::Less,
            CompareOp::Le => result != Ordering::Greater,
            CompareOp::Gt => result == Ordering::Greater,
            CompareOp::Ge => result != Ordering::Less,
        }
    }
}

/// A workaround for [associated const equality](https://github.com/rust-lang/rust/issues/92827).
///
/// This serves to have True / False values in the [`PyClass`] trait's `Frozen` type.
#[doc(hidden)]
pub mod boolean_struct {
    pub(crate) mod private {
        use super::*;

        /// A way to "seal" the boolean traits.
        pub trait Boolean {
            const VALUE: bool;
        }

        impl Boolean for True {
            const VALUE: bool = true;
        }
        impl Boolean for False {
            const VALUE: bool = false;
        }
    }

    pub struct True(());
    pub struct False(());
}

/// A trait which is used to describe whether a `#[pyclass]` is frozen.
#[doc(hidden)]
pub trait Frozen: boolean_struct::private::Boolean {}

impl Frozen for boolean_struct::True {}
impl Frozen for boolean_struct::False {}

mod tests {
    #[test]
    fn test_compare_op_matches() {
        use super::CompareOp;
        use std::cmp::Ordering;

        assert!(CompareOp::Eq.matches(Ordering::Equal));
        assert!(CompareOp::Ne.matches(Ordering::Less));
        assert!(CompareOp::Ge.matches(Ordering::Greater));
        assert!(CompareOp::Gt.matches(Ordering::Greater));
        assert!(CompareOp::Le.matches(Ordering::Equal));
        assert!(CompareOp::Lt.matches(Ordering::Less));
    }
}
