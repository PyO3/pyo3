// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python Iterator Interface.
//! Trait and support implementation for implementing iterators

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::err::PyResult;
use crate::{ffi, IntoPy, IntoPyPointer, PyClass, PyObject, Python};

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

/// Output of `__next__` which can either `yield` the next value in the iteration, or
/// `return` a value to raise `StopIteration` in Python.
///
/// See [`PyIterProtocol`](trait.PyIterProtocol.html) for an example.
pub enum IterNextOutput<T, U> {
    /// The value yielded by the iterator.
    Yield(T),
    /// The `StopIteration` object.
    Return(U),
}

pub type PyIterNextOutput = IterNextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterNextOutput {
    fn convert(self, _py: Python) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterNextOutput::Yield(o) => Ok(o.into_ptr()),
            IterNextOutput::Return(opt) => Err(crate::exceptions::PyStopIteration::new_err((opt,))),
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterNextOutput> for IterNextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterNextOutput> {
        match self {
            IterNextOutput::Yield(o) => Ok(IterNextOutput::Yield(o.into_py(py))),
            IterNextOutput::Return(o) => Ok(IterNextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterNextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterNextOutput> {
        match self {
            Some(o) => Ok(PyIterNextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterNextOutput::Return(py.None())),
        }
    }
}
