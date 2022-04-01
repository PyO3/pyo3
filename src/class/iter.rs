#![allow(deprecated)]
// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python Iterator Interface.
//! Trait and support implementation for implementing iterators

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::{PyClass, PyObject};

/// Python Iterator Interface.
///
/// Check [CPython doc](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_iter)
/// for more.
///
/// # Examples
/// The following example shows how to implement a simple Python iterator in Rust which yields
/// the integers 1 to 5, before raising `StopIteration("Ended")`.
///
/// ```rust
/// # #![allow(deprecated, elided_lifetimes_in_paths)]
/// use pyo3::class::iter::IterNextOutput;
/// use pyo3::prelude::*;
/// use pyo3::PyIterProtocol;
///
/// #[pyclass]
/// struct Iter {
///     count: usize,
/// }
///
/// #[pyproto]
/// impl PyIterProtocol for Iter {
///     fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<usize, &'static str> {
///         if slf.count < 5 {
///             slf.count += 1;
///             IterNextOutput::Yield(slf.count)
///         } else {
///             IterNextOutput::Return("Ended")
///         }
///     }
/// }
///
/// # Python::with_gil(|py| {
/// #     let inst = Py::new(py, Iter { count: 0 }).unwrap();
/// #     pyo3::py_run!(py, inst, "assert next(inst) == 1");
/// # }); // test of StopIteration is done in pytests/src/pyclasses.rs
/// ```
#[allow(unused_variables)]
#[deprecated(since = "0.16.0", note = "prefer `#[pymethods]` to `#[pyproto]`")]
pub trait PyIterProtocol<'p>: PyClass {
    fn __iter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterIterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __next__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterNextProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyIterIterProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyIterNextProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyIterNextOutput>;
}

py_unarys_func!(iter, PyIterIterProtocol, Self::__iter__);
py_unarys_func!(iternext, PyIterNextProtocol, Self::__next__);

pub use crate::pyclass::{IterNextOutput, PyIterNextOutput};
