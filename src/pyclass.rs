//! `PyClass` and related traits.
use crate::{
    callback::IntoPyCallbackOutput, ffi, impl_::pyclass::PyClassImpl, IntoPy, PyObject, PyResult,
    PyTypeInfo, Python,
};
use std::{cmp::Ordering, os::raw::c_int};

mod create_type_object;
mod gc;

pub(crate) use self::create_type_object::{create_type_object, PyClassTypeObject};
pub use self::gc::{PyTraverseError, PyVisit};

/// Types that can be used as Python classes.
///
/// The `#[pyclass]` attribute implements this trait for your Rust struct -
/// you shouldn't implement this trait directly.
#[allow(deprecated)]
#[cfg(feature = "gil-refs")]
pub trait PyClass: PyTypeInfo<AsRefTarget = crate::PyCell<Self>> + PyClassImpl {
    /// Whether the pyclass is frozen.
    ///
    /// This can be enabled via `#[pyclass(frozen)]`.
    type Frozen: Frozen;
}

/// Types that can be used as Python classes.
///
/// The `#[pyclass]` attribute implements this trait for your Rust struct -
/// you shouldn't implement this trait directly.
#[cfg(not(feature = "gil-refs"))]
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
    /// ```rust
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

/// Output of `__next__` which can either `yield` the next value in the iteration, or
/// `return` a value to raise `StopIteration` in Python.
///
/// Usage example:
///
/// ```rust
/// # #![allow(deprecated)]
/// use pyo3::prelude::*;
/// use pyo3::iter::IterNextOutput;
///
/// #[pyclass]
/// struct PyClassIter {
///     count: usize,
/// }
///
/// #[pymethods]
/// impl PyClassIter {
///     #[new]
///     pub fn new() -> Self {
///         PyClassIter { count: 0 }
///     }
///
///     fn __next__(&mut self) -> IterNextOutput<usize, &'static str> {
///         if self.count < 5 {
///             self.count += 1;
///             // Given an instance `counter`, First five `next(counter)` calls yield 1, 2, 3, 4, 5.
///             IterNextOutput::Yield(self.count)
///         } else {
///             // At the sixth time, we get a `StopIteration` with `'Ended'`.
///             //     try:
///             //         next(counter)
///             //     except StopIteration as e:
///             //         assert e.value == 'Ended'
///             IterNextOutput::Return("Ended")
///         }
///     }
/// }
/// ```
#[deprecated(since = "0.21.0", note = "Use `Option` or `PyStopIteration` instead.")]
pub enum IterNextOutput<T, U> {
    /// The value yielded by the iterator.
    Yield(T),
    /// The `StopIteration` object.
    Return(U),
}

/// Alias of `IterNextOutput` with `PyObject` yield & return values.
#[deprecated(since = "0.21.0", note = "Use `Option` or `PyStopIteration` instead.")]
#[allow(deprecated)]
pub type PyIterNextOutput = IterNextOutput<PyObject, PyObject>;

#[allow(deprecated)]
impl<T, U> IntoPyCallbackOutput<*mut ffi::PyObject> for IterNextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterNextOutput::Yield(o) => Ok(o.into_py(py).into_ptr()),
            IterNextOutput::Return(o) => {
                Err(crate::exceptions::PyStopIteration::new_err(o.into_py(py)))
            }
        }
    }
}

/// Output of `__anext__`.
///
/// <https://docs.python.org/3/reference/expressions.html#agen.__anext__>
#[deprecated(
    since = "0.21.0",
    note = "Use `Option` or `PyStopAsyncIteration` instead."
)]
pub enum IterANextOutput<T, U> {
    /// An expression which the generator yielded.
    Yield(T),
    /// A `StopAsyncIteration` object.
    Return(U),
}

/// An [IterANextOutput] of Python objects.
#[deprecated(
    since = "0.21.0",
    note = "Use `Option` or `PyStopAsyncIteration` instead."
)]
#[allow(deprecated)]
pub type PyIterANextOutput = IterANextOutput<PyObject, PyObject>;

#[allow(deprecated)]
impl<T, U> IntoPyCallbackOutput<*mut ffi::PyObject> for IterANextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterANextOutput::Yield(o) => Ok(o.into_py(py).into_ptr()),
            IterANextOutput::Return(o) => Err(crate::exceptions::PyStopAsyncIteration::new_err(
                o.into_py(py),
            )),
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
