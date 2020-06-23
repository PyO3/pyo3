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
    type Result: IntoPyCallbackOutput<IterNextOutput>;
}

#[derive(Default)]
pub struct PyIterMethods {
    pub tp_iter: Option<ffi::getiterfunc>,
    pub tp_iternext: Option<ffi::iternextfunc>,
}

#[doc(hidden)]
impl PyIterMethods {
    pub(crate) fn update_typeobj(&self, type_object: &mut ffi::PyTypeObject) {
        type_object.tp_iter = self.tp_iter;
        type_object.tp_iternext = self.tp_iternext;
    }
    pub fn set_iter<T>(&mut self)
    where
        T: for<'p> PyIterIterProtocol<'p>,
    {
        self.tp_iter = py_unarys_func!(PyIterIterProtocol, T::__iter__);
    }
    pub fn set_iternext<T>(&mut self)
    where
        T: for<'p> PyIterNextProtocol<'p>,
    {
        self.tp_iternext = py_unarys_func!(PyIterNextProtocol, T::__next__);
    }
}

pub struct IterNextOutput(Option<PyObject>);

impl IntoPyCallbackOutput<*mut ffi::PyObject> for IterNextOutput {
    fn convert(self, _py: Python) -> PyResult<*mut ffi::PyObject> {
        match self.0 {
            Some(o) => Ok(o.into_ptr()),
            None => Err(crate::exceptions::StopIteration::py_err(())),
        }
    }
}

impl<T> IntoPyCallbackOutput<IterNextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<IterNextOutput> {
        Ok(IterNextOutput(self.map(|o| o.into_py(py))))
    }
}
